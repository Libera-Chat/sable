//! Contains types used by [`Network`] to notify callers of state changes

use crate::network::state;
use crate::prelude::*;
use sable_macros::event_details;

use state::{HistoricMessageSource, HistoricMessageTarget, HistoricUser};

#[derive(Debug)]
pub struct WrongEventTypeError;

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
        pub user: HistoricUser,
        pub new_nick: Nickname,
    }

    /// A user's mode has changed
    struct UserModeChange {
        pub user: HistoricUser,
        pub added: UserModeSet,
        pub removed: UserModeSet,
        pub changed_by: HistoricMessageSource,
    }

    /// A user's away reason/status has changed
    struct UserAwayChange {
        pub user: HistoricUser,
        /// None iff the user was not away
        pub old_reason: Option<AwayReason>,
        /// None iff the user is no longer away
        pub new_reason: Option<AwayReason>,
    }

    /// A user has left the network
    struct UserQuit {
        pub user: HistoricUser,
        pub nickname: Nickname,
        pub message: String,
        pub memberships: Vec<state::Membership>,
    }

    /// A new connection has attached to an existing user
    struct NewUserConnection {
        pub user: HistoricUser,
        pub connection: state::UserConnection,
    }

    /// A client has disconnected, without the user quitting
    struct UserConnectionDisconnected {
        pub user: HistoricUser,
        pub connection: state::UserConnection,
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

    /// A user was kicked from a channel
    struct ChannelKick {
        pub membership: state::Membership,
        pub source: HistoricMessageSource,
        pub channel: state::Channel,
        pub user: HistoricUser,
        pub message: String,
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
        pub source: HistoricMessageSource,
        pub channel: state::Channel,
        pub old_name: ChannelName,
        pub new_name: ChannelName,
        pub message: String,
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
