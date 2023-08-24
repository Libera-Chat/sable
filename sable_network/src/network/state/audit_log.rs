use crate::prelude::*;

use serde::{Deserialize, Serialize};

/// An audit log category
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AuditLogCategory {
    General,
    NetworkBan,
    ServerKill,
}

/// An audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: AuditLogEntryId,
    pub timestamp: i64,
    pub category: AuditLogCategory,
    pub source_id: Option<UserId>,
    pub source_addr: Option<std::net::IpAddr>,
    pub source_str: String,
    pub action: String,
    pub target_id: Option<UserId>,
    pub target_str: Option<String>,
    pub target_duration: Option<i64>,
    pub reason: Option<String>,
}
