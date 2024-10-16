//! History storage and retrieval

use std::collections::HashMap;

use thiserror::Error;

use crate::history::HistoryLogEntry;
use crate::network::state::{HistoricMessageSourceId, HistoricMessageTargetId};
use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

pub enum HistoryRequest {
    Latest {
        to_ts: Option<i64>,
        limit: usize,
    },
    Before {
        from_ts: i64,
        limit: usize,
    },
    After {
        start_ts: i64,
        limit: usize,
    },
    Around {
        around_ts: i64,
        limit: usize,
    },
    Between {
        start_ts: i64,
        end_ts: i64,
        limit: usize,
    },
}

#[derive(Error, Debug)]
pub enum HistoryError {
    #[error("invalid target: {0:?}")]
    InvalidTarget(TargetId),
}

/// A backend implementation of [IRCv3 CHATHISTORY](https://ircv3.net/specs/extensions/chathistory)
pub trait HistoryService {
    /// Returns a list of list of history logs the given user has access to
    ///
    /// And the timestamp of the last known message in that log.
    fn list_targets(
        &self,
        user: UserId,
        after_ts: Option<i64>,
        before_ts: Option<i64>,
        limit: Option<usize>,
    ) -> HashMap<TargetId, i64>;

    fn get_entries(
        &self,
        user: UserId,
        target: TargetId,
        request: HistoryRequest,
    ) -> Result<impl Iterator<Item = &HistoryLogEntry>, HistoryError>;
}
