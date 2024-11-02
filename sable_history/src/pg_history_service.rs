use std::collections::HashMap;

use anyhow::{bail, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use futures::stream::{StreamExt, TryStreamExt};
use tokio::sync::Mutex;
use uuid::Uuid;

use sable_network::prelude::*;

use crate::schema::{channels, historic_users, messages};

/// Implementation of [`HistoryService`] backed PostgreSQL
pub struct PgHistoryService<'a> {
    database_connection: &'a Mutex<AsyncPgConnection>,
}

impl<'a> PgHistoryService<'a> {
    pub fn new(database_connection: &'a Mutex<AsyncPgConnection>) -> Self {
        Self {
            database_connection,
        }
    }
}

impl<'a> HistoryService for PgHistoryService<'a> {
    async fn list_targets(
        &self,
        _user: UserId,
        _after_ts: Option<i64>,
        _before_ts: Option<i64>,
        _limit: Option<usize>,
    ) -> HashMap<TargetId, i64> {
        // TODO: access control
        // TODO: after_ts, before_ts, limit
        match channels::dsl::channels
            .select((
                channels::dsl::id,
                sql::<diesel::pg::sql_types::Uuid>(
                    "SELECT MAX(id) FROM messages WHERE target_channel=channels.id",
                ),
            ))
            .load_stream(&mut *self.database_connection.lock().await)
            .await
        {
            Err(e) => {
                tracing::error!("Could not get history channels: {e}");
                return HashMap::new();
            }
            Ok(rows) => rows
                .map(|row| -> Result<(TargetId, i64)> {
                    let (channel_id, max_message_id): (i64, Uuid) = row?;
                    let channel =
                        TargetId::Channel(ChannelId::from(Snowflake::from(channel_id as u64)));

                    let Some(ts) = max_message_id.get_timestamp() else {
                        bail!("messages.id should be a UUID7, not {max_message_id}");
                    };
                    let (seconds, _) = ts.to_unix();
                    let Ok(seconds) = seconds.try_into() else {
                        bail!("message {max_message_id}'s UNIX timestamp is negative");
                    };
                    Ok((channel, seconds))
                })
                .try_collect()
                .await
                .unwrap_or_else(|e| {
                    tracing::error!("Could not read rows: {e}");
                    HashMap::new()
                }),
        }
    }

    async fn get_entries(
        &self,
        _user: UserId,
        target: TargetId,
        request: HistoryRequest,
    ) -> Result<impl IntoIterator<Item = HistoricalEvent>, HistoryError> {
        // TODO: access control
        let TargetId::Channel(channel_id) = target else {
            // TODO: PMs
            return Err(HistoryError::InvalidTarget(target));
        };

        let mut connection_lock = self.database_connection.lock().await;

        let db_channel_id = channel_id.as_u64() as i64;
        let channel = match channels::dsl::channels
            .find(db_channel_id)
            .select(crate::models::Channel::as_select())
            .first(&mut *connection_lock)
            .await
            .optional()
        {
            Ok(Some(channel)) => channel,
            Ok(None) => return Err(HistoryError::InvalidTarget(target)),
            Err(e) => {
                tracing::error!("Could not check if channel exists: {e}");
                return Err(HistoryError::InternalError(
                    "Could not check if channel exists".to_string(),
                ));
            }
        };

        let base_query = messages::dsl::messages
            .inner_join(historic_users::dsl::historic_users)
            .select((
                messages::dsl::id,
                messages::dsl::timestamp,
                messages::dsl::message_type,
                messages::dsl::text,
                historic_users::dsl::nick,
                historic_users::dsl::ident,
                historic_users::dsl::vhost,
                historic_users::dsl::account_name,
            ))
            .filter(messages::dsl::target_channel.eq(db_channel_id));
        Ok(match request {
            HistoryRequest::Latest { to_ts, limit } => {
                let limit = i64::min(10000, i64::try_from(limit).unwrap_or(i64::MAX));
                match to_ts {
                    Some(to_ts) => {
                        let to_ts = DateTime::from_timestamp(to_ts, 999_999)
                            .unwrap_or(DateTime::<Utc>::MIN_UTC)
                            .naive_utc();
                        Box::new(
                            base_query
                                .filter(messages::dsl::timestamp.gt(to_ts))
                                // total order, consistent across requests
                                .order((messages::dsl::timestamp.desc(), messages::dsl::id.desc()))
                                .limit(limit)
                                .load_stream(&mut *connection_lock),
                        )
                    }
                    None => Box::new(
                        base_query
                            // total order, consistent across requests
                            .order((messages::dsl::timestamp.desc(), messages::dsl::id.desc()))
                            .limit(limit)
                            .load_stream(&mut *connection_lock),
                    ),
                }
                .await
                .map_err(|e| {
                    tracing::error!("Could not query messages: {e}");
                    HistoryError::InternalError("Could not query messages".to_string())
                })?
                .map_ok(|row| make_historical_event(&channel, row))
                .try_collect::<Vec<_>>()
                .await
                .map_err(|e| {
                    tracing::error!("Could not parse messages: {e}");
                    HistoryError::InternalError("Could not parse message".to_string())
                })?
                .into_iter()
                .rev() // need to reverse *after* applying the SQL LIMIT
                .collect::<Vec<_>>()
            }
            HistoryRequest::Before { from_ts, limit } => {
                let limit = i64::min(10000, i64::try_from(limit).unwrap_or(i64::MAX));
                let from_ts = DateTime::from_timestamp(from_ts, 0)
                    .unwrap_or(DateTime::<Utc>::MAX_UTC)
                    .naive_utc();
                Box::new(
                    base_query
                        .filter(messages::dsl::timestamp.lt(from_ts))
                        // total order, consistent across requests
                        .order((messages::dsl::timestamp.desc(), messages::dsl::id.desc()))
                        .limit(limit)
                        .load_stream(&mut *connection_lock),
                )
                .await
                .map_err(|e| {
                    tracing::error!("Could not query messages: {e}");
                    HistoryError::InternalError("Could not query messages".to_string())
                })?
                .map_ok(|row| make_historical_event(&channel, row))
                .try_collect::<Vec<_>>()
                .await
                .map_err(|e| {
                    tracing::error!("Could not parse messages: {e}");
                    HistoryError::InternalError("Could not parse message".to_string())
                })?
                .into_iter()
                .rev() // need to reverse *after* applying the SQL LIMIT
                .collect::<Vec<_>>()
            }
            HistoryRequest::After { start_ts, limit } => {
                let limit = i64::min(10000, i64::try_from(limit).unwrap_or(i64::MAX));
                let start_ts = DateTime::from_timestamp(start_ts, 999_999)
                    .unwrap_or(DateTime::<Utc>::MIN_UTC)
                    .naive_utc();
                Box::new(
                    base_query
                        .filter(messages::dsl::timestamp.gt(start_ts))
                        // total order, consistent across requests
                        .order((messages::dsl::timestamp, messages::dsl::id))
                        .limit(limit)
                        .load_stream(&mut *connection_lock),
                )
                .await
                .map_err(|e| {
                    tracing::error!("Could not query messages: {e}");
                    HistoryError::InternalError("Could not query messages".to_string())
                })?
                .map_ok(|row| make_historical_event(&channel, row))
                .try_collect::<Vec<_>>()
                .await
                .map_err(|e| {
                    tracing::error!("Could not parse messages: {e}");
                    HistoryError::InternalError("Could not parse message".to_string())
                })?
            }
            HistoryRequest::Around { around_ts, limit } => {
                todo!("around")
            }
            HistoryRequest::Between {
                start_ts,
                end_ts,
                limit,
            } => {
                if start_ts <= end_ts {
                    let start_ts = DateTime::from_timestamp(start_ts, 999_999)
                        .unwrap_or(DateTime::<Utc>::MIN_UTC)
                        .naive_utc();
                    let end_ts = DateTime::from_timestamp(end_ts, 0)
                        .unwrap_or(DateTime::<Utc>::MAX_UTC)
                        .naive_utc();
                    let limit = i64::min(10000, i64::try_from(limit).unwrap_or(i64::MAX));
                    Box::new(
                        base_query
                            .filter(messages::dsl::timestamp.gt(start_ts))
                            .filter(messages::dsl::timestamp.lt(end_ts))
                            // total order, consistent across requests
                            .order((messages::dsl::timestamp, messages::dsl::id))
                            .limit(limit)
                            .load_stream(&mut *connection_lock),
                    )
                    .await
                    .map_err(|e| {
                        tracing::error!("Could not query messages: {e}");
                        HistoryError::InternalError("Could not query messages".to_string())
                    })?
                    .map_ok(|row| make_historical_event(&channel, row))
                    .try_collect::<Vec<_>>()
                    .await
                    .map_err(|e| {
                        tracing::error!("Could not parse messages: {e}");
                        HistoryError::InternalError("Could not parse message".to_string())
                    })?
                } else {
                    let start_ts = DateTime::from_timestamp(start_ts, 0)
                        .unwrap_or(DateTime::<Utc>::MAX_UTC)
                        .naive_utc();
                    let end_ts = DateTime::from_timestamp(end_ts, 999_999)
                        .unwrap_or(DateTime::<Utc>::MIN_UTC)
                        .naive_utc();
                    let limit = i64::min(10000, i64::try_from(limit).unwrap_or(i64::MAX));
                    Box::new(
                        base_query
                            .filter(messages::dsl::timestamp.gt(end_ts))
                            .filter(messages::dsl::timestamp.lt(start_ts))
                            // total order, consistent across requests
                            .order((messages::dsl::timestamp.desc(), messages::dsl::id.desc()))
                            .limit(limit)
                            .load_stream(&mut *connection_lock),
                    )
                    .await
                    .map_err(|e| {
                        tracing::error!("Could not query messages: {e}");
                        HistoryError::InternalError("Could not query messages".to_string())
                    })?
                    .map_ok(|row| make_historical_event(&channel, row))
                    .try_collect::<Vec<_>>()
                    .await
                    .map_err(|e| {
                        tracing::error!("Could not parse messages: {e}");
                        HistoryError::InternalError("Could not parse message".to_string())
                    })?
                    .into_iter()
                    .rev() // need to reverse *after* applying the SQL LIMIT
                    .collect::<Vec<_>>()
                }
            }
        })
    }
}

fn make_historical_event(
    channel: &crate::models::Channel,
    (id, timestamp, message_type, text, source_nick, source_ident, source_vhost, source_account): (
        uuid::Uuid,
        NaiveDateTime,
        crate::types::MessageType,
        String,
        String,
        String,
        String,
        Option<String>,
    ),
) -> HistoricalEvent {
    HistoricalEvent::Message {
        id: MessageId::new(id.try_into().expect("Message id is a non-v7 UUID")),
        timestamp: timestamp.and_utc().timestamp(),
        source: format!("{}!{}@{}", source_nick, source_ident, source_vhost),
        source_account,
        message_type: message_type.into(),
        target: channel.name.clone(), // assume it's the same
        text,
    }
}
