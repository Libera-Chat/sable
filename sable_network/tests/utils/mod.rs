use sable_network::network::{config::NetworkConfig, Network};
use serde::Serialize;
use std::collections::HashMap;

pub mod receiver;

pub fn empty_network_config() -> NetworkConfig {
    NetworkConfig {
        opers: Vec::new(),
        debug_mode: false,
        default_roles: HashMap::new(),
        alias_users: Vec::new(),
    }
}

pub fn empty_network() -> Network {
    Network::new(empty_network_config())
}

pub fn stringify<T: Serialize>(obj: &T) -> String {
    serde_json::to_string(obj).unwrap()
}
