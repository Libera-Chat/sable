use std::borrow::BorrowMut;
use std::collections::HashMap;

use diesel::dsl::sql;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use futures::stream::StreamExt;
use tokio::sync::Mutex;
use uuid::Uuid;

use sable_network::prelude::*;

use crate::schema::channels;

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
        user: UserId,
        after_ts: Option<i64>,
        before_ts: Option<i64>,
        limit: Option<usize>,
    ) -> HashMap<TargetId, i64> {
        // TODO: access control
        match diesel_async::RunQueryDsl::load_stream(
            channels::dsl::channels.select((
                channels::dsl::id,
                sql::<diesel::pg::sql_types::Uuid>(
                    "SELECT MAX(id) FROM messages WHERE target_channel=channels.id",
                ),
            )),
            &mut *self.database_connection.lock().await,
        )
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
        user: UserId,
        target: TargetId,
        request: HistoryRequest,
    ) -> Result<impl IntoIterator<Item = HistoryLogEntry>, HistoryError> {
        todo!();
        Ok(vec![])
    }
}
