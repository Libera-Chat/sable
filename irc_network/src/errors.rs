//! Defines errors returned by the other modules

use thiserror::Error;
use crate::*;

/// Types of error that can occur while looking up network objects
#[derive(Error,Debug)]
pub enum LookupError
{
    #[error("Wrong ID type")]
    WrongType,
    #[error("No such user id {0:?}")]
    NoSuchUser(UserId),
    #[error("No such user mode id {0:?}")]
    NoSuchUserMode(UserModeId),
    #[error("No user with mode id {0:?}")]
    NoUserForMode(UserModeId),
    #[error("No such channel id {0:?}")]
    NoSuchChannel(ChannelId),
    #[error("No such channel mode id {0:?}")]
    NoSuchChannelMode(ChannelModeId),
    #[error("No channel with mode id {0:?}")]
    NoChannelForMode(ChannelModeId),
    #[error("No such banlist id {0:?}")]
    NoSuchListMode(ListModeId),
    #[error("No channel mode corresponds to banlist id {0:?}")]
    NoModeForList(ListModeId),
    #[error("No such channel topic id {0:?}")]
    NoSuchChannelTopic(ChannelTopicId),
    #[error("No topic for channel id {0:?}")]
    NoTopicForChannel(ChannelId),
    #[error("No such membership id {0:?}")]
    NoSuchMembership(MembershipId),
    #[error("No such server {0:?}")]
    NoSuchServer(ServerId),
    #[error("No such nickname {0}")]
    NoSuchNick(String),
    #[error("Couldn't find nickname for user {0:?}")]
    NoNickForUser(UserId),
    #[error("No such channel name {0}")]
    NoSuchChannelName(String),
    #[error("No such message id {0:?}")]
    NoSuchMessage(MessageId),
    #[error("Connection id not found")]
    NoSuchConnectionId,
}

/// Convenience definition of a Result type used to look up network objects.
pub type LookupResult<T> = std::result::Result<T, LookupError>;

