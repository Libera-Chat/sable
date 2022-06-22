use crate::prelude::*;

use serde::{
    Serialize,
    Deserialize
};

/// An audit log category
#[derive(Debug,Clone,Copy,Serialize,Deserialize)]
pub enum AuditLogCategory
{
    General,
    NetworkBan,
    ServerKill,
}

/// A data field for an audit log entry
#[derive(Debug,Clone,Copy,Serialize,Deserialize)]
pub enum AuditLogField
{
    Source,
    ActionType,
    TargetUser,
    NetworkBanMask,
    NetworkBanDuration,
    Reason,
}

/// An audit log entry
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AuditLogEntry
{
    pub id: AuditLogEntryId,
    pub timestamp: i64,
    pub category: AuditLogCategory,
    pub fields: Vec<(AuditLogField, String)>
}
