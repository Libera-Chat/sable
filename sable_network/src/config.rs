use serde::Deserialize;
use std::{path::PathBuf, error::Error, fs::File, io::BufReader};

use crate::prelude::ConfigError;

#[derive(Debug,Deserialize)]
pub struct TlsConfig
{
    pub key_file: PathBuf,
    pub cert_file: PathBuf,
}

#[derive(Clone,Debug)]
pub struct TlsData
{
    pub key: Vec<u8>,
    pub cert_chain: Vec<Vec<u8>>,
}

impl TlsConfig
{
    pub fn load_from_disk(&self) -> Result<TlsData, Box<dyn Error>>
    {
        let cert_file = File::open(&self.cert_file)?;
        let mut cert_reader = BufReader::new(cert_file);
        let cert_chain = rustls_pemfile::certs(&mut cert_reader)?;

        let key_file = File::open(&self.key_file)?;
        let mut key_reader = BufReader::new(key_file);

        let server_key = rustls_pemfile::read_one(&mut key_reader)?;

        use rustls_pemfile::Item;

        let server_key = match server_key {
            Some(Item::RSAKey(key)) | Some(Item::PKCS8Key(key)) => Ok(key),
            Some(Item::X509Certificate(_)) | None => Err(ConfigError::FormatError("No private key in file".to_string()))
        }?;

        Ok(TlsData { key: server_key, cert_chain })
    }
}

