use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ListenerConfig {
    pub address: String,
    #[serde(default)]
    pub tls: bool,
}

#[derive(Debug, Deserialize)]
pub struct ClientServerConfig {
    pub listeners: Vec<ListenerConfig>,
}
