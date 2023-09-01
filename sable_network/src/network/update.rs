//! Contains types used by [`Network`] to notify callers of state changes

use crate::network::state;
use crate::prelude::*;
use sable_macros::event_details;

#[derive(Debug)]
pub struct WrongEventTypeError;

/// Info about a User at a point in time, in a form which can be stored for replay. This contains
/// both the [`state::User`] object itself and the user's nickname at the given time
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoricUser {
    pub user: state::User,
    pub nickname: Nickname,
}

/// Some state changes can originate from either users or servers; this enum is used in the
/// [`NetworkStateChange`] for those changes to describe the source of the change.
///
/// This roughly corresponds to "things that can go in the source field of a client protocol
/// message".
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HistoricMessageSource {
    Server(state::Server),
    User(HistoricUser),
    Unknown,
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

event_details!(

/// Emitted by the `Network` to signal that a change has happened which needs to be notified
/// or otherwise processed. These are distinct from the `Event`s which are input to the network;
/// one `Event` may cause the network to emit any number of state updates.
///
/// These objects are also stored in the server's chat history log, for replay to clients that
/// support it.
///
/// Note that the parameters are all copies of the state objects, as the originals may have already
/// been removed from the network state when the change is emitted, or when it is replayed from the
/// history log. Where possible, the consumer may want to call back to the `Network` to get wrapper
/// objects.
NetworkStateChange => {
    /// A new user has joined the network
    struct NewUser {
        pub user: HistoricUser,
    }

    ///  A user has changed nickname
    struct UserNickChange {
        pub user: state::User,
        pub old_nick: Nickname,
        pub new_nick: Nickname,
    }

    /// A user's mode has changed
    struct UserModeChange {
        pub user: HistoricUser,
        pub added: UserModeSet,
        pub removed: UserModeSet,
        pub changed_by: HistoricMessageSource,
    }

    /// A user has left the network
    struct UserQuit {
        pub user: HistoricUser,
        pub nickname: Nickname,
        pub message: String,
        pub memberships: Vec<state::Membership>,
    }

    /// Many users have left the network
    struct BulkUserQuit {
        pub items: Vec<UserQuit>,
    }

    /// A channel's mode has changed
    struct ChannelModeChange {
        pub channel: state::Channel,
        pub added: ChannelModeSet,
        pub removed: ChannelModeSet,
        pub key_change: OptionChange<ChannelKey>,
        pub changed_by: HistoricMessageSource,
    }

    /// A channel's topic has changed
    struct ChannelTopicChange {
        pub channel: state::Channel,
        pub topic: state::ChannelTopic,
        pub new_text: String,
        pub setter: HistoricMessageSource,
        pub timestamp: i64,
    }

    /// A list-type mode (+bqeI) has been added to a channel
    struct ListModeAdded {
        pub channel: state::Channel,
        pub list_type: ListModeType,
        pub pattern: Pattern,
        pub set_by: HistoricMessageSource,
    }

    /// A list-type mode has been removed from a channel
    struct ListModeRemoved {
        pub channel: state::Channel,
        pub list_type: ListModeType,
        pub pattern: Pattern,
        pub removed_by: HistoricMessageSource,
    }

    /// A membership flag (+ov) has been added to or removed from a channel members
    struct MembershipFlagChange {
        pub membership: state::Membership,
        pub user: HistoricUser,
        pub channel: state::Channel,
        pub added: MembershipFlagSet,
        pub removed: MembershipFlagSet,
        pub changed_by: HistoricMessageSource,
    }

    /// A user has joined a channel
    struct ChannelJoin {
        pub membership: state::Membership,
        pub user: HistoricUser,
        pub channel: state::Channel,
    }

    /// A user has left a channel
    struct ChannelPart {
        pub membership: state::Membership,
        pub user: HistoricUser,
        pub channel: state::Channel,
        pub message: String,
    }

    /// A user has been invited to a channel
    struct ChannelInvite {
        pub invite: state::ChannelInvite,
        pub source: HistoricMessageSource,
        pub user: HistoricUser,
        pub channel: state::Channel
    }

    /// A channel's name has changed
    struct ChannelRename {
        pub channel: state::Channel,
        pub old_name: ChannelName,
        pub new_name: ChannelName,
    }

    /// A message has been sent to a user or channel
    struct NewMessage {
        pub message: state::Message,
        pub source: HistoricMessageSource,
        pub target: HistoricMessageTarget,
    }

    /// A new server has joined the network
    struct NewServer {
        pub server: state::Server,
    }

    /// A server has left the network
    struct ServerQuit {
        pub server: state::Server,
    }

    /// An entry has been added to the network audit log
    struct NewAuditLogEntry {
        pub entry: state::AuditLogEntry,
    }

    /// A user has logged into or out of an account
    struct UserLoginChange {
        pub user: HistoricUser,
        pub old_account: Option<state::Account>,
        pub new_account: Option<state::Account>,
    }

    /// The current services node has changed
    struct ServicesUpdate {
        pub new_state: Option<state::ServicesData>,
    }

    /// A delimiter event to denote that an Event has been completely processed
    struct EventComplete { }
});

/// Trait to be implemented by an object which wants to be notified of network state updates
///
/// An instance of this is passed to `Network::apply` to receive all updates caused by that
/// operation.
///
/// This primarily exists to avoid the network state library depending on tokio or other async
/// runtime for channel types.
pub trait NetworkUpdateReceiver {
    /// Notify the receiver of a network state change
    fn notify_update(&self, update: NetworkStateChange, source_event: &Event);
}

use std::convert::Into;

/// Helper to make sending updates easier
pub(crate) trait NetworkUpdateHelper {
    fn notify(&self, update: impl Into<NetworkStateChange>, source_event: &Event);
}

impl<T: NetworkUpdateReceiver + ?Sized> NetworkUpdateHelper for T {
    fn notify(&self, update: impl Into<NetworkStateChange>, source_event: &Event) {
        self.notify_update(update.into(), source_event);
    }
}
