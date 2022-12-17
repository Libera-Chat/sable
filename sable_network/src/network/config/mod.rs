use serde::{Serialize,Deserialize};
use serde_with::serde_as;
use std::collections::HashMap;
use super::state;

#[serde_as]
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NetworkConfig
{
    pub opers: Vec<OperConfig>,
    pub debug_mode: bool,

    #[serde_as(as = "HashMap<_, state::HumanReadableChannelAccessSet>")]
    pub default_roles: HashMap<state::ChannelRoleName, state::ChannelAccessSet>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct OperConfig
{
    pub name: String,
    pub hash: String,
}

impl NetworkConfig
{
    pub fn new() -> Self
    {
        Self {
            opers: Vec::new(),
            debug_mode: false,
            default_roles: HashMap::new(),
        }
    }
}