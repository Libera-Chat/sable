use crate::prelude::*;

use serde::{Deserialize, Serialize};

use ban::*;

/// A network ban
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkBan {
    pub id: NetworkBanId,
    pub created_by: EventId,

    pub match_type: BanMatchType,
    pub pattern: crate::chert::NodeBoolean,
    pub action: NetworkBanAction,

    pub timestamp: i64,
    pub expires: i64,
    pub reason: String,
    pub oper_reason: Option<String>,
    pub setter_info: String,
}
