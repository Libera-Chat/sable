use crate::id::*;
use crate::validated::*;
use serde::{
    Serialize,
    Deserialize
};

/// A server
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Server
{
    pub id: ServerId,
    pub epoch: EpochId,
    pub name: ServerName,
    pub last_ping: i64,
    pub introduced_by: EventId,
}