use std::borrow::BorrowMut;
use std::collections::HashMap;

use diesel::dsl::sql;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use futures::stream::{StreamExt, TryStreamExt};
use tokio::sync::Mutex;
use uuid::Uuid;

use sable_network::prelude::*;

use crate::schema::channels;
use crate::schema::messages;

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
            Ok(rows) => {
                rows.map(|row| row.expect("Could not deserialize row"))
                    .map(
                        |(channel_id, max_message_id): (i64, Uuid)| -> (TargetId, i64) {
                            let (seconds, _) = max_message_id
                                .get_timestamp()
                                .expect("messages.id is not a UUID7")
                                .to_unix();
                            (
                                TargetId::Channel(ChannelId::from(Snowflake::from(
                                    u64::try_from(channel_id).expect("channel id is negative"),
                                ))),
                                seconds
                                    .try_into()
                                    .expect("message's UNIX timestamp is negative"),
                            )
                        },
                    )
                    .collect()
                    .await
            }
        }
    }

    async fn get_entries(
        &self,
        _user: UserId,
        target: TargetId,
        request: HistoryRequest,
    ) -> Result<impl IntoIterator<Item = HistoryLogEntry>, HistoryError> {
        // TODO: access control
        let TargetId::Channel(channel_id) = target else {
            // TODO: PMs
            return Err(HistoryError::InvalidTarget(target));
        };

        let mut connection_lock = self.database_connection.lock().await;

        let db_channel_id = i64::try_from(channel_id.as_u64()).expect("channel id overflows u64");
        if channels::dsl::channels
            .find(db_channel_id)
            .select(crate::models::Channel::as_select())
            .first(&mut *connection_lock)
            .await
            .optional()
            .expect("Could not check if channel exists")
            .is_none()
        {
            return Err(HistoryError::InvalidTarget(target));
        }

        match request {
            HistoryRequest::Latest { to_ts, limit } => {
                let limit = i64::min(10000, i64::try_from(limit).unwrap_or(i64::MAX));
                Ok(match to_ts {
                    Some(to_ts) => {
                        // Lowest UUIDv7 corresponding to the timestamp
                        let to_uuid = uuid::Builder::from_unix_timestamp_millis(
                            u64::try_from(to_ts)
                                .unwrap_or(u64::MIN) // floor timestamps to Epoch
                                .saturating_mul(1000),
                            &[u8::MIN; 10],
                        )
                        .into_uuid();
                        Box::new(
                            messages::dsl::messages
                                .filter(messages::dsl::target_channel.eq(db_channel_id))
                                .filter(messages::dsl::id.lt(to_uuid))
                                .order(messages::dsl::id.desc())
                                .limit(limit)
                                .load_stream(&mut *connection_lock),
                        )
                    }
                    None => Box::new(
                        messages::dsl::messages
                            .filter(messages::dsl::target_channel.eq(db_channel_id))
                            .order(messages::dsl::id.desc())
                            .limit(limit)
                            .load_stream(&mut *connection_lock),
                    ),
                }
                .await
                .expect("could not query messages")
                .map(|row| {
                    let row: crate::models::Message = match row {
                        Ok(row) => row,
                        Err(e) => return Err(e),
                    };
                    /*
                    let crate::models::Message {
                        id,
                        source_user,
                        target_channel,
                        text,
                    } = row?;
                    }
                    Ok(HistoryLogEntry {
                        id: (),
                        details: NetworkStateChange::NewMessage(update::NewMessage {
                            message: (),
                            source: (),
                            target: (),
                        }),
                        source_event: (),
                        timestamp: (),
                    })
                    */
                    Ok((|| -> HistoryLogEntry { todo!() })())
                })
                .try_collect::<Vec<_>>()
                .await
                .expect("could not parse all records"))
            }
            HistoryRequest::Before { from_ts, limit } => {
                todo!("before")
            }
            HistoryRequest::After { start_ts, limit } => {
                todo!("after")
            }
            HistoryRequest::Around { around_ts, limit } => {
                todo!("around")
            }
            HistoryRequest::Between {
                start_ts,
                end_ts,
                limit,
            } => {
                todo!("between")
            }
        }
    }
}
