//! Contains types used by [`Network`](crate::Network) to notify callers of state changes

use crate::state;
use crate::*;
use ircd_macros::event_details;
use irc_strings::matches::Pattern;

#[derive(Debug)]
pub struct WrongEventTypeError;

event_details!(

/// Emitted by the `Network` to signal that a change has happened which needs to be notified
/// or otherwise processed. These are distinct from the `Event`s which are input to the network;
/// one `Event` may cause the network to emit any number of state updates.
///
/// Note that the parameters are all copies of the state objects, as the originals may have already
/// been removed from the network state. Where possible, the consumer may want to call back to the
///  `Network` to get wrapper objects.
NetworkStateChange => {
    /// A new user has joined the network
    struct NewUser {
        pub user: UserId,
    }

    ///  A user has changed nickname
    struct UserNickChange {
        pub user: UserId,
        pub old_nick: Nickname,
        pub new_nick: Nickname,
    }

    /// A user's mode has changed
    struct UserModeChange {
        pub user_id: UserId,
        pub added: UserModeSet,
        pub removed: UserModeSet,
        pub changed_by: ObjectId,
    }

    /// A user has left the network
    struct UserQuit {
        pub user: state::User,
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
        pub channel: ChannelId,
        pub added: ChannelModeSet,
        pub removed: ChannelModeSet,
        pub key_change: OptionChange<ChannelKey>,
        pub changed_by: ObjectId,
    }

    /// A channel's topic has changed
    struct ChannelTopicChange {
        pub topic: ChannelTopicId,
        pub new_text: String,
        pub setter: ObjectId,
        pub timestamp: i64,
    }

    /// A list-type mode (+bqeI) has been added to a channel
    struct ListModeAdded {
        pub channel: ChannelId,
        pub list: ListModeId,
        pub list_type: ListModeType,
        pub pattern: Pattern,
        pub set_by: ObjectId,
    }

    /// A list-type mode has been removed from a channel
    struct ListModeRemoved {
        pub channel: ChannelId,
        pub list: ListModeId,
        pub list_type: ListModeType,
        pub pattern: Pattern,
        pub removed_by: ObjectId,
    }

    /// A membership flag (+ov) has been added to or removed from a channel members
    struct MembershipFlagChange {
        pub membership: MembershipId,
        pub added: MembershipFlagSet,
        pub removed: MembershipFlagSet,
        pub changed_by: ObjectId,
    }

    /// A user has joined a channel
    struct ChannelJoin {
        pub membership: MembershipId,
    }

    /// A user has left a channel
    struct ChannelPart {
        pub membership: state::Membership,
        // This needs to be here explicitly because when the last user leaves a channel,
        // the channel won't exist any more to look up from the membership details
        pub channel_name: ChannelName,
        pub message: String,
    }

    /// A user has been invited to a channel
    struct ChannelInvite {
        pub id: InviteId,
        pub source: UserId,
    }

    /// A channel's name has changed
    struct ChannelRename {
        pub id: ChannelId,
        pub old_name: ChannelName,
        pub new_name: ChannelName,
    }

    /// A message has been sent to a user or channel
    struct NewMessage {
        pub message: MessageId,
    }

    /// A new server has joined the network
    struct NewServer {
        pub id: ServerId,
    }

    /// A server has left the network
    struct ServerQuit {
        pub server: state::Server,
    }

    /// An entry has been added to the network audit log
    struct NewAuditLogEntry {
        pub id: AuditLogEntryId,
    }
});

/// Trait to be implemented by an object which wants to be notified of network state updates
///
/// An instance of this is passed to `Network::apply` to receive all updates caused by that
/// operation.
///
/// This primarily exists to avoid the network state library depending on tokio or other async
/// runtime for channel types.
pub trait NetworkUpdateReceiver
{
    /// Notify the receiver of a network state change
    fn notify_update(&self, update: NetworkStateChange);
}

use std::convert::Into;

/// Helper to make sending updates easier
pub(crate) trait NetworkUpdateHelper
{
    fn notify(&self, update: impl Into<NetworkStateChange>);
}

impl<T: NetworkUpdateReceiver + ?Sized> NetworkUpdateHelper for T
{
    fn notify(&self, update: impl Into<NetworkStateChange>)
    {
        self.notify_update(update.into());
    }
}