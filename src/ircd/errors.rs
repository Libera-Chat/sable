use thiserror::Error;
use crate::ircd::*;
use irc::*;
use async_std::channel;

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
    #[error("No such nickname {0}")]
    NoSuchNick(String),
    #[error("No such channel name {0}")]
    NoSuchChannelName(String),
    #[error("Connection id not found")]
    NoSuchConnectionId,
}

pub type LookupResult<T> = std::result::Result<T, LookupError>;

#[derive(Debug,Error)]
pub enum HandlerError {
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Connection error: {0}")]
    ConnectionError(#[from]ConnectionError),
    #[error("Object lookup failed: {0}")]
    LookupError(#[from] LookupError),
    #[error("Mismatched object ID type")]
    WrongIdType(#[from] WrongIdTypeError),
}

impl From<&str> for HandlerError
{
    fn from(msg: &str) -> Self { Self::InternalError(msg.to_string()) }
}

pub type HandleResult = Result<(), HandlerError>;

#[derive(Error,Debug)]
pub enum ConnectionError {
    #[error("Connection closed")]
    Closed,
    #[error("I/O Error: {0}")]
    IoError(#[from]std::io::Error),
    #[error("Couldn't send to control channel: {0}")]
    ControlSendError(#[from] channel::SendError<connection::ConnectionControl>),
    #[error("Send queue full")]
    SendQueueFull,
}

