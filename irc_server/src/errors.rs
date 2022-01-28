use thiserror::Error;
use irc_network::*;
use client_listener::ConnectionError;

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

