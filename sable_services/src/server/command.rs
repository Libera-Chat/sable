use super::*;
use sable_network::prelude::*;
use thiserror::Error;

#[derive(Debug,Error)]
pub enum CommandError
{
    #[error("{0}")]
    LookupError(#[from] LookupError),
    #[error("{0:?}")]
    ErrorResponse(RemoteServerResponse),
    #[error("{0}")]
    DatabaseError(#[from] crate::database::DatabaseError),
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

impl From<&str> for CommandError
{
    fn from(value: &str) -> Self {
        Self::UnknownError(value.to_owned())
    }
}

impl From<String> for CommandError
{
    fn from(value: String) -> Self {
        Self::UnknownError(value)
    }
}

impl From<RemoteServerResponse> for CommandError
{
    fn from(value: RemoteServerResponse) -> Self {
        Self::ErrorResponse(value)
    }
}

pub type CommandResult = Result<RemoteServerResponse, CommandError>;

mod user_commands;
mod channel_commands;