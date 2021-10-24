use thiserror::Error;
use crate::ircd::*;

#[derive(Error,Debug)]
pub enum LookupError
{
    #[error("Wrong ID type")]
    WrongType,
    #[error("No such user id {0:?}")]
    NoSuchUser(UserId),
    #[error("No such channel id {0:?}")]
    NoSuchChannel(ChannelId),
    #[error("No such channel mode id {0:?}")]
    NoSuchChannelMode(CModeId),
    #[error("No channel with mode id {0:?}")]
    NoChannelForMode(CModeId),
    #[error("No such membership id {0:?}")]
    NoSuchMembership(MembershipId),
    #[error("No such nickname {0}")]
    NoSuchNick(String),
    #[error("No such channel name {0}")]
    NoSuchChannelName(String),
    #[error("Connection id not found")]
    NoSuchConnectionId,
}

pub type LookupResult<T> = std::result::Result<T, LookupError>;

