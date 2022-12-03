use super::*;

/// A channel-related permission error
#[derive(Debug)]
pub enum ChannelPermissionError
{
    /// The user isn't in a channel, and needs to be in order to be allowed the operation
    UserNotOnChannel,
    /// Channel operator privileges are required
    UserNotOp,
    /// User is banned from the channel
    UserIsBanned,
    /// User cannot send for some other reason
    CannotSendToChannel,
    /// Channel is invite-only
    InviteOnlyChannel,
    /// User hasn't provided the right channel key
    BadChannelKey,
    /// Channel isn't registered (and needs to be)
    NotRegistered,
    /// User doesn't have access to the registered channel
    NoAccess,
}

/// A user-related permission error
#[derive(Debug)]
pub enum UserPermissionError
{
    /// User is not an oper
    NotOper,
    /// That user mode can't be set directly
    ReadOnlyUmode,
    /// User isn't logged in (and needs to be)
    NotLoggedIn,
}

#[derive(Debug)]
pub enum PermissionError
{
    Channel(ChannelName, ChannelPermissionError),
    User(UserPermissionError),
    InternalError(Box<dyn std::error::Error>),
}

impl From<LookupError> for PermissionError
{
    fn from(e: LookupError) -> Self { Self::InternalError(Box::new(e)) }
}