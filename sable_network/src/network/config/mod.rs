use serde::{Serialize,Deserialize};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NetworkConfig
{
    pub opers: Vec<OperConfig>,
    pub debug_mode: bool,
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
        }
    }
}