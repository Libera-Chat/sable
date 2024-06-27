//! Contains types used by [`Network`] to notify callers of state changes

use crate::network::state;
use crate::prelude::*;
use sable_macros::event_details;

use state::{HistoricMessageSourceId, HistoricMessageTargetId};

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
        pub user: HistoricUserId,
    }

    ///  A user has changed nickname
    struct UserNickChange {
        pub user: HistoricUserId,
        pub new_nick: Nickname,
    }

    /// A user's mode has changed
    struct UserModeChange {
        pub user: HistoricUserId,
        pub added: UserModeSet,
        pub removed: UserModeSet,
        pub changed_by: HistoricMessageSourceId,
    }

    /// A user's away reason/status has changed
    struct UserAwayChange {
        pub user: HistoricUserId,
        /// None iff the user was not away
        pub old_reason: Option<AwayReason>,
        /// None iff the user is no longer away
        pub new_reason: Option<AwayReason>,
    }

    /// A user has left the network
    struct UserQuit {
        pub user: HistoricUserId,
        pub nickname: Nickname,
        pub message: String,
        pub memberships: Vec<state::Membership>,
    }

    /// A new connection has attached to an existing user
    struct NewUserConnection {
        pub user: HistoricUserId,
        pub connection: UserConnectionId,
    }

    /// A client has disconnected, without the user quitting
    struct UserConnectionDisconnected {
        pub user: HistoricUserId,
        pub connection: state::UserConnection,
    }

    /// A channel's mode has changed
    struct ChannelModeChange {
        pub channel: ChannelId,
        pub added: ChannelModeSet,
        pub removed: ChannelModeSet,
        pub key_change: OptionChange<ChannelKey>,
        pub changed_by: HistoricMessageSourceId,
    }

    /// A channel's topic has changed
    struct ChannelTopicChange {
        pub channel: ChannelId,
        pub topic: ChannelTopicId,
        pub new_text: String,
        pub setter: HistoricMessageSourceId,
        pub timestamp: i64,
    }

    /// A list-type mode (+bqeI) has been added to a channel
    struct ListModeAdded {
        pub channel: ChannelId,
        pub list_type: ListModeType,
        pub pattern: Pattern,
        pub set_by: HistoricMessageSourceId,
    }

    /// A list-type mode has been removed from a channel
    struct ListModeRemoved {
        pub channel: ChannelId,
        pub list_type: ListModeType,
        pub pattern: Pattern,
        pub removed_by: HistoricMessageSourceId,
    }

    /// A membership flag (+ov) has been added to or removed from a channel members
    struct MembershipFlagChange {
        pub membership: MembershipId,
        pub user: HistoricUserId,
        pub added: MembershipFlagSet,
        pub removed: MembershipFlagSet,
        pub changed_by: HistoricMessageSourceId,
    }

    /// A user has joined a channel
    struct ChannelJoin {
        pub membership: MembershipId,
        pub user: HistoricUserId,
    }

    /// A user was kicked from a channel
    struct ChannelKick {
        pub membership: state::Membership,
        pub source: HistoricMessageSourceId,
        pub user: HistoricUserId,
        pub message: String,
    }

    /// A user has left a channel
    struct ChannelPart {
        pub membership: state::Membership,
        pub user: HistoricUserId,
        pub message: String,
    }

    /// A user has been invited to a channel
    struct ChannelInvite {
        pub invite: InviteId,
        pub source: HistoricMessageSourceId,
        pub user: HistoricUserId,
    }

    /// A channel's name has changed
    struct ChannelRename {
        pub source: HistoricMessageSourceId,
        pub channel: ChannelId,
        pub old_name: ChannelName,
        pub new_name: ChannelName,
        pub message: String,
    }

    /// A message has been sent to a user or channel
    struct NewMessage {
        pub message: MessageId,
        pub source: HistoricMessageSourceId,
        pub target: HistoricMessageTargetId,
    }

    /// A new server has joined the network
    struct NewServer {
        pub server: ServerId,
    }

    /// A server has left the network
    struct ServerQuit {
        pub server: state::Server,
    }

    /// An entry has been added to the network audit log
    struct NewAuditLogEntry {
        pub entry: AuditLogEntryId,
    }

    /// A user has logged into or out of an account
    struct UserLoginChange {
        pub user: HistoricUserId,
        pub old_account: Option<AccountId>,
        pub new_account: Option<AccountId>,
    }

    /// The current services node has changed
    struct ServicesUpdate {
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
