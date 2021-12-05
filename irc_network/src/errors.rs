use thiserror::Error;
use crate::*;

#[derive(Error,Debug)]
pub enum LookupError
{
    #[error("Wrong ID type")]
    WrongType,
    #[error("No such user id {0:?}")]
    NoSuchUser(UserId),
    #[error("No such user mode id {0:?}")]
    NoSuchUserMode(UModeId),
    #[error("No user with mode id {0:?}")]
    NoUserForMode(UModeId),
    #[error("No such channel id {0:?}")]
    NoSuchChannel(ChannelId),
    #[error("No such channel mode id {0:?}")]
    NoSuchChannelMode(CModeId),
    #[error("No channel with mode id {0:?}")]
    NoChannelForMode(CModeId),
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

pub type LookupResult<T> = std::result::Result<T, LookupError>;

