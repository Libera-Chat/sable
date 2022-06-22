use crate::prelude::*;

use serde::{
    Serialize,
    Deserialize
};

/// A K:line
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct KLine
{
    pub id: NetworkBanId,
    pub user: Pattern,
    pub host: Pattern,
    pub timestamp: i64,
    pub expires: i64,
    pub reason: String,
    pub oper_reason: Option<String>,
    pub setter_info: String,
}
