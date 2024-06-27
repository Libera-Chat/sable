use crate::{id::*, network::*, validated::*};

/// Info about a User at a point in time, in a form which can be stored for replay.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoricUser {
    pub id: UserId,
    pub nickname: Nickname,
    pub user: Username,
    pub visible_host: Hostname,
    pub realname: Realname,
    pub away_reason: Option<AwayReason>,
    pub account: Option<Nickname>,
}

impl wrapper::WrappedUser for HistoricUser {
    fn id(&self) -> crate::id::UserId {
        self.id
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
            visible_host: user.visible_host,
            realname: user.realname,
            away_reason: user.away_reason,
        }
    }
}

/// Some state changes can originate from either users or servers; this enum is used in the
/// [`NetworkStateChange`] for those changes to describe the source of the change.
///
/// This roughly corresponds to "things that can go in the source field of a client protocol
/// message".
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HistoricMessageSource {
    User(HistoricUser),
    Server(state::Server),
    Unknown,
}

impl HistoricMessageSource {
    /// Return the [`HistoricUser`] if it's a user source
    pub fn user(&self) -> Option<&HistoricUser> {
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
pub enum HistoricMessageTarget {
    User(HistoricUser),
    Channel(state::Channel),
    Unknown,
}
