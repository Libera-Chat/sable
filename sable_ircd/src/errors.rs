use thiserror::Error;
use sable_network::prelude::*;
use client_listener::ConnectionError;

/// An error that could occur when handling a network state change
#[derive(Debug,Error)]
pub enum HandlerError
{
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

impl<T> From<tokio::sync::mpsc::error::TrySendError<T>> for HandlerError
{
    fn from(err: tokio::sync::mpsc::error::TrySendError<T>) -> Self { Self::InternalError(err.to_string()) }
}

impl From<std::fmt::Error> for HandlerError
{
    fn from(e: std::fmt::Error) -> Self { Self::InternalError(e.to_string()) }
}

pub type HandleResult = Result<(), HandlerError>;

