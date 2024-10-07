use super::state;
use crate::validated::*;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub opers: Vec<OperConfig>,
    pub debug_mode: bool,

    #[serde_as(as = "HashMap<_, state::HumanReadableChannelAccessSet>")]
    pub default_roles: HashMap<state::ChannelRoleName, state::ChannelAccessSet>,

    pub alias_users: Vec<AliasUser>,

    pub object_expiry: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasUser {
    pub nick: Nickname,
    pub user: Username,
    pub host: Hostname,
    pub realname: Realname,

    pub command_alias: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperConfig {
    pub name: String,
    pub hash: String,
}

impl NetworkConfig {
    pub fn new() -> Self {
        Self {
            opers: Vec::new(),
            debug_mode: false,
            default_roles: HashMap::new(),
            alias_users: Vec::new(),
            object_expiry: 0,
        }
    }
}
