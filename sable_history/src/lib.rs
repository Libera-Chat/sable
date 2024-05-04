//! History storage and retrieval

use std::collections::HashMap;

use sable_network::history::HistoryLogEntry;
use sable_network::prelude::update::{HistoricMessageSource, HistoricMessageTarget};
use sable_network::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

impl TryFrom<&HistoricMessageSource> for TargetId {
    type Error = ();

    fn try_from(value: &HistoricMessageSource) -> Result<Self, Self::Error> {
        match value {
            HistoricMessageSource::Server(_) => Err(()), // Is that okay?
            HistoricMessageSource::User(user) => Ok(TargetId::User(user.user.id)),
            HistoricMessageSource::Unknown => Err(()),
        }
    }
}
impl TryFrom<&HistoricMessageTarget> for TargetId {
    type Error = ();

    fn try_from(value: &HistoricMessageTarget) -> Result<Self, Self::Error> {
        match value {
            HistoricMessageTarget::User(user) => Ok(TargetId::User(user.user.id)),
            HistoricMessageTarget::Channel(channel) => Ok(TargetId::Channel(channel.id)),
            HistoricMessageTarget::Unknown => Err(()),
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
    ) -> impl Iterator<Item = &HistoryLogEntry>;
}

pub mod local_history;

mod build_data {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
