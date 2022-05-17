use super::*;
use std::collections::HashMap;
use strum::IntoEnumIterator;
use itertools::Itertools;
use serde::{
    Serialize,
    Deserialize
};

#[derive(Debug,Serialize,Deserialize)]
pub struct CapabilityRepository
{
    supported_caps: HashMap<String, ClientCapability>,
    all_caps: String,
}

impl CapabilityRepository
{
    pub fn new() -> Self
    {
        let supported_caps: HashMap<String, ClientCapability> = ClientCapability::iter().map(|c| (c.name().to_string(), c)).collect();
        let all_caps = supported_caps.keys().join(" ");

        Self {
            supported_caps,
            all_caps,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=&ClientCapability>
    {
        self.supported_caps.values()
    }

    pub fn supported_caps(&self) -> &str
    {
        &self.all_caps
    }

    pub fn find(&self, name: &str) -> Option<ClientCapability>
    {
        self.supported_caps.get(name).map(|c| *c)
    }
}