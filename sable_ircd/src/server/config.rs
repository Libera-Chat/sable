use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ListenerConfig {
    pub address: String,
    #[serde(default)]
    pub tls: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClientServerConfig {
    pub listeners: Vec<ListenerConfig>,
}
