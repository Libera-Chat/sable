//! Contains definitions of the various configuration files and items required
//! in order to run a network sync node

use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::{fs::File, io::BufReader, net::SocketAddr, path::Path, path::PathBuf};

use rustls::{Certificate, PrivateKey};

/// Configuration of a peer in the gossip network
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerConfig {
    pub(crate) name: String,
    pub(crate) address: SocketAddr,
    pub(crate) fingerprint: String,
}

/// Configuration of the gossip network
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncConfig {
    pub(crate) peers: Vec<PeerConfig>,
    pub(crate) fanout: usize,

    pub(crate) ca_file: PathBuf,
}

/// Configuration for this server's node in the gossip network
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeConfig {
    pub(crate) listen_addr: SocketAddr,
    pub(crate) cert_file: PathBuf,
    pub(crate) key_file: PathBuf,
}

/// Errors that could happen when loading or processing a config
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error on {1}: {0}")]
    IoError(std::io::Error, PathBuf),
    #[error("JSON parse error in {1}: {0}")]
    JsonError(serde_json::Error, PathBuf),
    #[error("Invalid address specifier in {1}: {0}")]
    AddrParseError(std::net::AddrParseError, PathBuf),
    #[error("{1}: {0}")]
    FormatError(String, PathBuf),
    #[error("Missing field {0} in {1}")]
    MissingField(String, PathBuf),
}

impl SyncConfig {
    /// Load the network configuration from a given file path
    pub fn load_file<P: AsRef<Path> + Copy>(filename: P) -> Result<Self, ConfigError> {
        let file = File::open(filename)
            .map_err(|e| ConfigError::IoError(e, filename.as_ref().to_owned()))?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)
            .map_err(|e| ConfigError::JsonError(e, filename.as_ref().to_owned()))?)
    }

    /// Load and return the CA certificate for the network from the referenced
    /// file path
    pub fn load_ca_cert(&self) -> Result<Certificate, ConfigError> {
        let ca_file =
            File::open(&self.ca_file).map_err(|e| ConfigError::IoError(e, self.ca_file.clone()))?;
        let mut ca_reader = BufReader::new(ca_file);
        let ca_data = rustls_pemfile::certs(&mut ca_reader)
            .map_err(|e| ConfigError::IoError(e, self.ca_file.clone()))?
            .pop()
            .ok_or_else(|| {
                ConfigError::FormatError(
                    "No certificate in CA file".to_string(),
                    self.ca_file.clone(),
                )
            })?;

        Ok(Certificate(ca_data))
    }
}

impl NodeConfig {
    /// Load the node configuration from a given file path
    pub fn load_file<P: AsRef<Path> + Copy>(filename: P) -> Result<Self, ConfigError> {
        let file = File::open(filename)
            .map_err(|e| ConfigError::IoError(e, filename.as_ref().to_owned()))?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)
            .map_err(|e| ConfigError::JsonError(e, filename.as_ref().to_owned()))?)
    }

    /// Load and return the client certificate and private key for this node
    /// from the referenced file path
    pub fn load_cert_and_keys(&self) -> Result<(Vec<Certificate>, PrivateKey), ConfigError> {
        let cert_file = File::open(&self.cert_file)
            .map_err(|e| ConfigError::IoError(e, self.cert_file.clone()))?;
        let mut cert_reader = BufReader::new(cert_file);
        let cert_chain = rustls_pemfile::certs(&mut cert_reader)
            .map_err(|e| ConfigError::IoError(e, self.cert_file.clone()))?
            .into_iter()
            .map(Certificate)
            .collect();

        let key_file = File::open(&self.key_file)
            .map_err(|e| ConfigError::IoError(e, self.key_file.clone()))?;
        let mut key_reader = BufReader::new(key_file);
        let client_key = rustls_pemfile::rsa_private_keys(&mut key_reader)
            .map_err(|e| ConfigError::IoError(e, self.key_file.clone()))?
            .pop()
            .ok_or_else(|| {
                ConfigError::FormatError(
                    "No private key in file".to_string(),
                    self.key_file.clone(),
                )
            })?;

        Ok((cert_chain, PrivateKey(client_key)))
    }
}
