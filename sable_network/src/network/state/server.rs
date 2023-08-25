use crate::{id::*, validated::*};

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    /// Server flags
    #[derive(Serialize,Deserialize)]
    pub struct ServerFlags : u64
    {
        const DEBUG = 0x01;
    }
}

/// A server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: ServerId,
    pub epoch: EpochId,
    pub name: ServerName,
    pub last_ping: i64,
    pub flags: ServerFlags,
    pub version: String,
}
