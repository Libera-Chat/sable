use thiserror::Error;
use irc_network::*;
use tokio::sync::mpsc::error::TrySendError;

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
    #[error("Internal error")]
    InternalError,
    #[error("Send queue full")]
    SendQueueFull,
}


impl<T> From<TrySendError<T>> for ConnectionError
{
    fn from(e: TrySendError<T>) -> Self
    {
        match e
        {
            TrySendError::Full(_) => Self::SendQueueFull,
            TrySendError::Closed(_) => Self::Closed
        }
    }
}