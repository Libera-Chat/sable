//! History storage and retrieval

use std::collections::HashMap;
use std::future::Future;
use std::num::NonZeroUsize;

use thiserror::Error;

use crate::network::state::{HistoricMessageSourceId, HistoricMessageTargetId, MessageType};
use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TargetId {
    User(UserId),
    Channel(ChannelId),
}

impl From<UserId> for TargetId {
    fn from(value: UserId) -> Self {
        TargetId::User(value)
    }
}

impl From<ChannelId> for TargetId {
    fn from(value: ChannelId) -> Self {
        TargetId::Channel(value)
    }
}

impl TryFrom<&HistoricMessageSourceId> for TargetId {
    type Error = ();

    fn try_from(value: &HistoricMessageSourceId) -> Result<Self, Self::Error> {
        match value {
            HistoricMessageSourceId::Server(_) => Err(()), // Is that okay?
            HistoricMessageSourceId::User(user) => Ok(TargetId::User(*user.user())),
            HistoricMessageSourceId::Unknown => Err(()),
        }
    }
}
impl TryFrom<&HistoricMessageTargetId> for TargetId {
    type Error = ();

    fn try_from(value: &HistoricMessageTargetId) -> Result<Self, Self::Error> {
        match value {
            HistoricMessageTargetId::User(user) => Ok(TargetId::User(*user.user())),
            HistoricMessageTargetId::Channel(channel) => Ok(TargetId::Channel(*channel)),
            HistoricMessageTargetId::Unknown => Err(()),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HistoryRequest {
    Latest {
        to_ts: Option<i64>,
        limit: NonZeroUsize,
    },
    Before {
        from_ts: i64,
        limit: NonZeroUsize,
    },
    After {
        start_ts: i64,
        limit: NonZeroUsize,
    },
    Around {
        around_ts: i64,
        limit: NonZeroUsize,
    },
    Between {
        start_ts: i64,
        end_ts: i64,
        limit: NonZeroUsize,
    },
}

#[derive(Error, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HistoryError {
    #[error("invalid target: {0:?}")]
    InvalidTarget(TargetId),
    #[error("internal server error: {0:?}")]
    InternalError(String),
}

/// A backend implementation of [IRCv3 CHATHISTORY](https://ircv3.net/specs/extensions/chathistory)
pub trait HistoryService {
    /// Returns a list of list of history logs the given user has access to
    ///
    /// And the timestamp of the last known message in that log.
    ///
    /// `after_ts` and `before_ts` are matched against that timestamp,
    /// and they are ordered by that timestamp in ascending order before
    /// applying the `limit`.
    ///
    /// In pseudo-SQL, this means:
    ///
    /// ```text
    /// SELECT target_id, last_timestamp
    /// FROM (
    ///     SELECT target_id, MAX(timestamp)
    ///     FROM entries
    ///     GROUP BY target_id
    /// )
    /// WHERE :before_ts < last_timestamp AND last_timestamp < :after_ts
    /// ORDER BY last_timestamp
    /// LIMIT :limit
    /// ```
    fn list_targets(
        &self,
        user: UserId,
        after_ts: Option<i64>,
        before_ts: Option<i64>,
        limit: Option<NonZeroUsize>,
    ) -> impl Future<Output = HashMap<TargetId, i64>> + Send;

    fn get_entries(
        &self,
        user: UserId,
        target: TargetId,
        request: HistoryRequest,
    ) -> impl Future<Output = Result<impl IntoIterator<Item = HistoricalEvent> + Send, HistoryError>>
           + Send;
}

/// A more concrete representation of `sable_ircd`'s `HistoryItem`, with all its fields
/// inflated to strings that will be sent to the client
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HistoricalEvent {
    Message {
        id: MessageId,
        timestamp: i64,
        source: String,
        source_account: Option<String>,
        /// If `None`, it should be replaced by the recipient's current nick
        target: Option<String>,
        message_type: MessageType,
        text: String,
    },
}
