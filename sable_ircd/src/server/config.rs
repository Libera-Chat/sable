use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize, Clone)]
pub struct ListenerConfig {
    pub address: String,
    #[serde(default)]
    pub tls: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RawServerInfo {
    pub motd: Option<PathBuf>,
    pub admin: Option<AdminInfo>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RawClientServerConfig {
    pub listeners: Vec<ListenerConfig>,
    #[serde(flatten)]
    pub info_paths: RawServerInfo,
    #[serde(default)]
    pub monitor: MonitorConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerInfoStrings {
    pub motd: Option<Vec<String>>, // Linewise to not repeatedly split
    pub admin_info: Option<AdminInfo>,
    pub info: Vec<String>,
}

impl ServerInfoStrings {
    pub fn load(raw_info: &RawServerInfo) -> Result<ServerInfoStrings, ConfigProcessingError> {
        Ok(Self {
            motd: Self::get_info(&raw_info.motd, "motd")?
                .map(|file| file.lines().map(|v| v.to_string()).collect()),
            admin_info: raw_info.admin.clone(),
            info: include_str!("../../info.txt")
                .lines()
                .map(|v| v.to_string())
                .collect(),
        })
    }

    fn get_info(
        path: &Option<PathBuf>,
        name: &str,
    ) -> Result<Option<String>, ConfigProcessingError> {
        match path {
            Some(real_path) => Ok(Some(Self::read(real_path, name)?)),
            None => Ok(None),
        }
    }

    fn read(path: &PathBuf, name: &str) -> Result<String, ConfigProcessingError> {
        fs::read_to_string(path).map_err(|err| ConfigProcessingError {
            reason: format!("Unable to read info {name:?} from {path:?}: {err}"),
        })
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)] // Dont let typos into the config
pub struct AdminInfo {
    pub server_location: Option<String>,
    pub description: Option<String>,
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MonitorConfig {
    /// Maximum number of active MONITORs per client connection. Default to 100
    #[serde(default = "default_max_monitors_per_connection")]
    pub max_per_connection: u16,
    // TODO: add a maximum per user and/or per account, specific limits for
    // authenticated vs unauthenticated users, etc.
}

impl Default for MonitorConfig {
    fn default() -> MonitorConfig {
        MonitorConfig {
            max_per_connection: 64,
        }
    }
}

fn default_max_monitors_per_connection() -> u16 {
    MonitorConfig::default().max_per_connection
}

#[derive(Debug)]
pub struct ClientServerConfig {
    pub listeners: Vec<ListenerConfig>,
    pub info_strings: ServerInfoStrings,
    pub monitor: MonitorConfig,
}

#[derive(Debug, Error)]
#[error("Unable to process config: {reason}")]
pub struct ConfigProcessingError {
    reason: String,
}
