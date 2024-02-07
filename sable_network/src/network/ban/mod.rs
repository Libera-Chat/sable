use crate::{id::*, validated::*};

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use thiserror::Error;

//mod user_details;
//pub use user_details::*;

mod repository;
pub use repository::*;

/// The set of user information that's available to a pre-registration network ban pattern
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

/// The set of user information that's available to a pre-SASL-authentication network ban pattern
#[derive(Debug, Clone, chert::ChertStruct)]
pub struct PreSaslBanSettings {
    pub ip: IpAddr,
    pub tls: bool,
}

/// Actions that can be applied by a network ban
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum NetworkBanAction {
    /// Refuse new connections that match these criteria. The boolean parameter
    /// determines whether existing connections that match will also be disconnected.
    RefuseConnection(bool),
    /// Require that new connections matching these criteria log in to an account
    /// before registration. The boolean parameter determines whether existing matching
    /// connections that are not logged in to an account will be disconnected.
    RequireSasl(bool),
    /// Refuse new connections instantly, without allowing exemptions from other config entries
    /// (equivalent to legacy D:line). Only makes sense for a ban that matches only on
    /// IP address; the other information won't be present at immediate-disconnection time.
    DisconnectEarly,
}

/// Error type denoting an invalid ban mask was supplied
#[derive(Debug, Clone, Error)]
#[error("Invalid ban mask")]
pub struct InvalidBanMask;

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
