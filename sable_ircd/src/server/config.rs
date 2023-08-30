use std::path::PathBuf;
use std::{fmt::Display, fs};

use serde::{Deserialize, Serialize};

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
pub struct Infos {
    pub motd: Option<String>, // Linewise to not repeatedly split
}

impl Infos {
    pub fn load(paths: &InfoPaths) -> Result<Infos, ConfigProcessingError> {
        Ok(Self {
            motd: Self::get_info(&paths.motd, "motd")?,
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
    pub infos: Infos,
}

#[derive(Debug)]
pub struct ConfigProcessingError {
    reason: String,
}

impl std::error::Error for ConfigProcessingError {}

impl Display for ConfigProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Unable to process config: {}", self.reason))
    }
}
