use serde_json;
use serde::Deserialize;
use thiserror::Error;

use std::{
    fs::File,
    io::BufReader,
    path::Path,
    path::PathBuf,
    net::SocketAddr,
};

use rustls_pemfile;
use rustls::{
    Certificate,
    PrivateKey,
};

#[derive(Clone,Debug,Deserialize)]
pub struct PeerConfig
{
    pub(crate) name: String,
    pub(crate) address: SocketAddr,
}

#[derive(Debug,Deserialize)]
pub struct NetworkConfig
{
    pub(crate) peers: Vec<PeerConfig>,
    pub(crate) fanout: usize,

    pub(crate) ca_file: PathBuf,
}

#[derive(Debug,Deserialize)]
pub struct NodeConfig
{
    pub(crate) listen_addr: SocketAddr,
    pub(crate) cert_file: PathBuf,
    pub(crate) key_file: PathBuf,
}

#[derive(Debug,Error)]
pub enum ConfigError
{
    #[error("I/O error: {0}")]
    IoError(#[from]std::io::Error),
    #[error("JSON parse error: {0}")]
    JsonError(#[from]serde_json::Error),
    #[error("Invalid address specifier: {0}")]
    AddrParseError(#[from]std::net::AddrParseError),
    #[error("{0}")]
    FormatError(String),
    #[error("Missing field: {0}")]
    MissingField(String)
}

impl NetworkConfig
{
    pub fn load_file<P: AsRef<Path>>(filename: P) -> Result<Self, ConfigError>
    {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn load_ca_cert(&self) -> Result<Certificate, ConfigError>
    {
        let ca_file = File::open(&self.ca_file)?;
        let mut ca_reader = BufReader::new(ca_file);
        let ca_data = rustls_pemfile::certs(&mut ca_reader)?
                                        .pop()
                                        .ok_or(ConfigError::FormatError("No certificate in CA file".to_string()))?;

        Ok(Certificate(ca_data))
    }
}

impl NodeConfig
{
    pub fn load_file<P: AsRef<Path>>(filename: P) -> Result<Self, ConfigError>
    {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn load_cert_and_keys(&self) -> Result<(Vec<Certificate>, PrivateKey), ConfigError>
    {
        let cert_file = File::open(&self.cert_file)?;
        let mut cert_reader = BufReader::new(cert_file);
        let cert_chain = rustls_pemfile::certs(&mut cert_reader)?.into_iter().map(|v| Certificate(v)).collect();

        let key_file = File::open(&self.key_file)?;
        let mut key_reader = BufReader::new(key_file);
        let client_key = rustls_pemfile::rsa_private_keys(&mut key_reader)?
                                        .pop()
                                        .ok_or(ConfigError::FormatError("No private key in file".to_string()))?;

        Ok((cert_chain, PrivateKey(client_key)))
    }
}