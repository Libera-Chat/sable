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
pub struct InfoPaths {
    pub motd: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClientServerConfig {
    pub listeners: Vec<ListenerConfig>,
    #[serde(flatten)]
    pub info_paths: InfoPaths,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerInfoStrings {
    pub motd: Option<Vec<String>>, // Linewise to not repeatedly split
}

impl ServerInfoStrings {
    pub fn load(paths: &InfoPaths) -> Result<ServerInfoStrings, ConfigProcessingError> {
        Ok(Self {
            motd: Self::get_info(&paths.motd, "motd")?
                .and_then(|file| Some(file.lines().map(|v| v.to_string()).collect())),
        })
    }

    fn get_info(
        path: &Option<PathBuf>,
        name: &str,
    ) -> Result<Option<String>, ConfigProcessingError> {
        match path {
            Some(real_path) => Ok(Some(Self::read(&real_path, name)?)),
            None => Ok(None),
        }
    }

    fn read(path: &PathBuf, name: &str) -> Result<String, ConfigProcessingError> {
        fs::read_to_string(path).or_else(|err| {
            Err(ConfigProcessingError {
                reason: format!("Unable to read info {name:?} from {path:?}: {err}"),
            })
        })
    }
}

pub struct ProcessedCSConfig {
    pub listeners: Vec<ListenerConfig>,
    pub info_strings: ServerInfoStrings,
}

#[derive(Debug, Error)]
#[error("Unable to process config: {reason}")]
pub struct ConfigProcessingError {
    reason: String,
}
