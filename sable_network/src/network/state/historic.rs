use crate::{id::*, network::*, validated::*};

/// Info about a User at a point in time, in a form which can be stored for replay.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoricUser {
    pub id: UserId,
    pub serial: u32,
    pub nickname: Nickname,
    pub user: Username,
    pub visible_host: Hostname,
    pub realname: Realname,
    pub away_reason: Option<AwayReason>,
    pub account: Option<Nickname>,

    /// The time until which this historic user state was accurate - if None
    /// then this data is current.
    pub timestamp: Option<i64>,
}

impl wrapper::WrappedUser for HistoricUser {
    fn id(&self) -> crate::id::UserId {
        self.id
    }

    fn historic_id(&self) -> HistoricUserId {
        HistoricUserId::new(self.id, self.serial)
    }

    fn nick(&self) -> Nickname {
        self.nickname
    }

    fn user(&self) -> &Username {
        &self.user
    }

    fn visible_host(&self) -> &Hostname {
        &self.visible_host
    }

    fn realname(&self) -> &Realname {
        &self.realname
    }

    fn away_reason(&self) -> Option<&AwayReason> {
        self.away_reason.as_ref()
    }

    fn nuh(&self) -> String {
        format!(
            "{}!{}@{}",
            self.nick().value(),
            self.user.value(),
            self.visible_host.value()
        )
    }

    fn account_name(&self) -> Option<Nickname> {
        self.account
    }
}

impl HistoricUser {
    pub fn new(user: &super::User, network: &Network) -> Self {
        Self {
            nickname: network.infallible_nick_for_user(user.id),
            account: user
                .account
                .and_then(|id| network.account(id).ok())
                .map(|acc| acc.name()),
            user: user.user,
            id: user.id,
            serial: user.serial,
            visible_host: user.visible_host,
            realname: user.realname,
            away_reason: user.away_reason,
            timestamp: None,
        }
    }
}

/// Some state changes can originate from either users or servers; this enum is used in the
/// [`NetworkStateChange`] for those changes to describe the source of the change.
///
/// This roughly corresponds to "things that can go in the source field of a client protocol
/// message".
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HistoricMessageSourceId {
    User(HistoricUserId),
    Server(ServerId),
    Unknown,
}

impl HistoricMessageSourceId {
    /// Return the [`HistoricUserId`] if it's a user source
    pub fn user(&self) -> Option<&HistoricUserId> {
        match self {
            Self::User(user) => Some(user),
            _ => None,
        }
    }
}

/// Some messages can be targeted at either a user or a channel; this enum is used in the
/// [`NetworkStateChange`] for those changes to describe the target in a way that can be
/// replayed later
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HistoricMessageTargetId {
    User(HistoricUserId),
    Channel(ChannelId),
    Unknown,
}

impl HistoricMessageTargetId {
    /// Return the [`HistoricUserId`] if it's a user source
    pub fn user(&self) -> Option<&HistoricUserId> {
        match self {
            Self::User(user) => Some(user),
            _ => None,
        }
    }
}
