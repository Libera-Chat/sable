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
    struct NewUser {
        pub user: UserId,
    }
    struct UserNickChange {
        pub user: UserId,
        pub old_nick: Nickname,
        pub new_nick: Nickname,
    }
    struct UserModeChange {
        pub user_id: UserId,
        pub mode_id: UserModeId,
        pub added: UserModeSet,
        pub removed: UserModeSet,
        pub changed_by: ObjectId,
    }
    struct UserQuit {
        pub user: state::User,
        pub nickname: Nickname,
        pub message: String,
        pub memberships: Vec<state::Membership>,
    }
    struct BulkUserQuit {
        pub items: Vec<UserQuit>,
    }
    struct ChannelModeChange {
        pub channel: ChannelId,
        pub mode: ChannelModeId,
        pub added: ChannelModeSet,
        pub removed: ChannelModeSet,
        pub key_change: OptionChange<ChannelKey>,
        pub changed_by: ObjectId,
    }
    struct ChannelTopicChange {
        pub topic: ChannelTopicId,
        pub new_text: String,
        pub setter: ObjectId,
        pub timestamp: i64,
    }
    struct ListModeAdded {
        pub channel: ChannelId,
        pub list: ListModeId,
        pub list_type: ListModeType,
        pub pattern: Pattern,
        pub set_by: ObjectId,
    }
    struct ListModeRemoved {
        pub channel: ChannelId,
        pub list: ListModeId,
        pub list_type: ListModeType,
        pub pattern: Pattern,
        pub removed_by: ObjectId,
    }
    struct MembershipFlagChange {
        pub membership: MembershipId,
        pub added: MembershipFlagSet,
        pub removed: MembershipFlagSet,
        pub changed_by: ObjectId,
    }
    struct ChannelJoin {
        pub membership: MembershipId,
    }
    struct ChannelPart {
        pub membership: state::Membership,
        // This needs to be here explicitly because when the last user leaves a channel,
        // the channel won't exist any more to look up from the membership details
        pub channel_name: ChannelName,
        pub message: String,
    }
    struct ChannelInvite {
        pub id: InviteId,
        pub source: UserId,
    }
    struct ChannelRename {
        pub id: ChannelId,
        pub old_name: ChannelName,
        pub new_name: ChannelName,
    }
    struct NewMessage {
        pub message: MessageId,
    }
    struct NewServer {
        pub id: ServerId,
    }
    struct ServerQuit {
        pub server: state::Server,
    }
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