use crate::{id::*, validated::*};

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use thiserror::Error;

//mod user_details;
//pub use user_details::*;

mod repository;
pub use repository::*;

/// Describes when a network ban will be matched, and which set of information is available to it
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BanMatchType {
    /// Matches immediately before registration, and also against existing connections
    /// when newly added - approximately equivalent to old K:line. Match fields are those in
    /// [`PreRegistrationBanSettings`].
    PreRegistration,
    /// Matches as soon as a connection is received, before processing any messages,
    /// and also against existing connections when added - approximately equivalent to old D:line.
    /// Match fields are those in [`NewConnectionBanSettings`].
    NewConnection,
    /// Matches when SASL authentication is initiated. Match fields are those in [`PreSaslBanSettings`].
    PreSasl,
}

/// The set of user information that's available to a `PreRegistration` network ban pattern
#[derive(Debug, Clone, chert::ChertStruct)]
pub struct PreRegistrationBanSettings {
    #[chert(as_ref=str)]
    pub nick: Nickname,
    #[chert(as_ref=str)]
    pub user: Username,
    #[chert(as_ref=str)]
    pub host: Hostname,
    #[chert(as_ref=str)]
    pub realname: Realname,
    pub ip: IpAddr,
    pub user_param_1: String,
    pub user_param_2: String,
    pub tls: bool,
}

/// The set of user information that's available to a `NewConnection` network ban pattern
#[derive(Debug, Clone, chert::ChertStruct)]
pub struct NewConnectionBanSettings {
    pub ip: IpAddr,
    pub tls: bool,
}

/// The set of user information that's available to a `PreSasl` network ban pattern
#[derive(Debug, Clone, chert::ChertStruct)]
pub struct PreSaslBanSettings {
    pub ip: IpAddr,
    pub tls: bool,
    pub mechanism: String,
}

/// Actions that can be applied by a network ban
#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NetworkBanAction {
    /// Refuse new connections that match these criteria. The boolean parameter
    /// determines whether existing connections that match will also be disconnected.
    RefuseConnection(bool),
    /// Require that new connections matching these criteria log in to an account
    /// before registration. The boolean parameter determines whether existing matching
    /// connections that are not logged in to an account will be disconnected.
    RequireSasl(bool),
    /// Prevent matching connections from using SASL authentication
    DenySasl,
}

/// Error type denoting an invalid ban mask was supplied
#[derive(Debug, Clone, Error)]
#[error("Invalid ban mask")]
pub struct InvalidBanPattern;

/// Error type denoting that a duplicate ban was provided
#[derive(Debug, Clone, Error)]
#[error("Duplicate network ban")]
pub struct DuplicateNetworkBan {
    /// The ID of the pre-existing ban
    pub existing_id: NetworkBanId,
    pub ban: crate::network::state::NetworkBan,
}

//#[cfg(test)]
//mod test;
