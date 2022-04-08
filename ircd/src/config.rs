use crate::*;
use std::net::SocketAddr;

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

#[derive(Debug,Deserialize)]
pub struct ListenerConfig
{
    pub address: String,
    #[serde(default)]
    pub tls: bool,
}

#[derive(Clone,Debug,serde::Deserialize)]
pub struct AuthorisedFingerprint
{
    pub name: String,
    pub fingerprint: String,
}

#[derive(Clone,Debug,serde::Deserialize)]
pub struct ManagementConfig
{
    pub address: SocketAddr,
    pub client_ca: PathBuf,
    pub authorised_fingerprints: Vec<AuthorisedFingerprint>,
}

#[derive(Debug,Deserialize)]
pub struct ServerConfig
{
    pub server_id: ServerId,
    pub server_name: ServerName,

    pub management: ManagementConfig,
    pub console_address: Option<String>,

    pub listeners: Vec<ListenerConfig>,

    pub tls_config: TlsConfig,
    pub node_config: NodeConfig,
}

impl ServerConfig
{
    pub fn load_file<P: AsRef<Path>>(filename: P) -> Result<Self, ConfigError>
    {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }
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

impl ManagementConfig
{
    pub fn load_client_ca(&self) -> std::io::Result<Vec<u8>>
    {
        let ca_file = File::open(&self.client_ca)?;
        let mut ca_reader = BufReader::new(ca_file);
        Ok(rustls_pemfile::certs(&mut ca_reader)?.remove(0))
    }
}

pub fn load_network_config(filename: impl AsRef<Path>) -> Result<irc_network::config::NetworkConfig, ircd_sync::ConfigError>
{
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

