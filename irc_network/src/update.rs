use crate::state;
use crate::validated::*;
use crate::flags::*;
use crate::id::*;
use ircd_macros::event_details;

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
        pub mode_id: UModeId,
        pub added: UserModeSet,
        pub removed: UserModeSet,
        pub changed_by: ObjectId,
    }
    struct UserQuit {
        pub user: state::User,
        pub message: String,
        pub memberships: Vec<state::Membership>,
    }
    struct BulkUserQuit {
        pub items: Vec<UserQuit>,
    }
    struct ChannelModeChange {
        pub channel: ChannelId,
        pub mode: CModeId,
        pub added: ChannelModeSet,
        pub removed: ChannelModeSet,
        pub changed_by: ObjectId,
    }
    struct ChannelPermissionChange {
        pub membership: MembershipId,
        pub added: ChannelPermissionSet,
        pub removed: ChannelPermissionSet,
        pub changed_by: ObjectId,
    }
    struct ChannelJoin {
        pub membership: MembershipId,
    }
    struct ChannelPart {
        pub membership: state::Membership,
        pub message: String,
    }
    struct NewMessage {
        pub message: MessageId,
    }
    struct ServerQuit {
        pub server: state::Server,
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