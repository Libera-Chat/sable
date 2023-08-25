use anyhow::Context;
use serde::Deserialize;
use std::{fs::File, io::BufReader, path::PathBuf};

use crate::prelude::ConfigError;

#[derive(Debug, Deserialize)]
pub struct TlsConfig {
    pub key_file: PathBuf,
    pub cert_file: PathBuf,
}

#[derive(Clone, Debug)]
pub struct TlsData {
    pub key: Vec<u8>,
    pub cert_chain: Vec<Vec<u8>>,
}

impl TlsConfig {
    pub fn load_from_disk(&self) -> Result<TlsData, anyhow::Error> {
        let cert_file = File::open(&self.cert_file)
            .with_context(|| format!("Could not open certificate {}", self.cert_file.display()))?;
        let mut cert_reader = BufReader::new(cert_file);
        let cert_chain = rustls_pemfile::certs(&mut cert_reader)
            .with_context(|| format!("Could not parse certificate {}", self.cert_file.display()))?;

        let key_file = File::open(&self.key_file)
            .with_context(|| format!("Could not open private key {}", self.key_file.display()))?;
        let mut key_reader = BufReader::new(key_file);

        let server_key = rustls_pemfile::read_one(&mut key_reader)
            .with_context(|| format!("Could not parse private key {}", self.key_file.display()))?;

        use rustls_pemfile::Item;

        let server_key = match server_key {
            Some(Item::RSAKey(key)) | Some(Item::PKCS8Key(key)) => Ok(key),
            Some(Item::X509Certificate(_)) | None => Err(ConfigError::FormatError(
                "No private key in file".to_string(),
                self.key_file.clone(),
            )),
        }?;

        Ok(TlsData {
            key: server_key,
            cert_chain,
        })
    }
}
