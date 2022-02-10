use thiserror::Error;
use serde::{Serialize,Deserialize};
use tokio::sync::mpsc::error::{
    SendError,
    TrySendError
};

/// An error that might occur on a single connection.
#[derive(Error,Debug,Serialize,Deserialize)]
pub enum ConnectionError
{
    #[error("Connection closed")]
    Closed,
    #[error("I/O Error: {0}")]
    IoError(String),
    #[error("Internal error")]
    InternalError,
    #[error("Send queue full")]
    SendQueueFull,
}

/// An error that might occur when configuring a listener.
#[derive(Error,Debug,Serialize,Deserialize)]
pub enum ListenerError
{
    #[error("TLS requested with no TLS config")]
    NoTlsConfig,
    #[error("I/O Error: {0}")]
    IoError(String),
    #[error("Error communicating with listener process")]
    CommunicationError,
}

impl From<std::io::Error> for ListenerError
{
    fn from(e: std::io::Error) -> Self
    {
        Self::IoError(e.to_string())
    }
}

impl<T> From<SendError<T>> for ListenerError
{
    fn from(_: SendError<T>) -> Self { Self::CommunicationError }
}

impl From<std::io::Error> for ConnectionError
{
    fn from(e: std::io::Error) -> Self
    {
        Self::IoError(e.to_string())
    }
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