use crate::*;
use crate::strip_comments::StripComments;
use std::net::SocketAddr;
use std::collections::HashMap;
use tracing_core::{
    LevelFilter,
};

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

#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
#[serde(rename_all="lowercase")]
pub enum BuiltinLogTarget
{
    Stdout,
    Stderr,
}

#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
#[serde(untagged)]
pub enum LogTarget
{
    File { filename: PathBuf },
    Builtin(BuiltinLogTarget),
}

#[derive(Clone,Copy,Debug,serde::Serialize,serde::Deserialize)]
#[serde(rename_all ="lowercase")]
pub enum LogLevel
{
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Off,
}

#[derive(Clone,Copy,Debug,serde::Serialize,serde::Deserialize)]
pub enum LogFormat
{
    Full,
    Compact,
    Pretty
}

#[derive(Clone,Copy,Debug,serde::Serialize,serde::Deserialize)]
#[serde(untagged)]
pub enum LogCategory
{
    General,
}

#[derive(Clone,Debug,serde::Deserialize)]
pub struct LogEntry
{
    pub target: LogTarget,
//    #[serde(default)]
//    pub categories: Vec<LogCategory>,
    #[serde(default)]
    pub modules: Vec<String>,
    pub level: Option<LogLevel>,
}

#[derive(Clone,Debug,serde::Deserialize)]
#[serde(rename_all="kebab-case")]
pub struct LoggingConfig
{
    pub dir: PathBuf,
    pub stdout: Option<PathBuf>,
    pub stderr: Option<PathBuf>,
    pub pidfile: Option<PathBuf>,
    pub default_level: Option<LogLevel>,
    pub module_levels: HashMap<String, LogLevel>,
    pub targets: Vec<LogEntry>,
    pub console_address: Option<std::net::SocketAddr>,
}

#[derive(Debug,Deserialize)]
pub struct ServerConfig
{
    pub server_id: ServerId,
    pub server_name: ServerName,

    pub management: ManagementConfig,

    pub listeners: Vec<ListenerConfig>,

    pub tls_config: TlsConfig,
    pub node_config: NodeConfig,

    pub log: LoggingConfig,
}

impl ServerConfig
{
    pub fn load_file<P: AsRef<Path>>(filename: P) -> Result<Self, ConfigError>
    {
        let file = File::open(filename)?;
        let reader = StripComments::new(file);
        Ok(serde_json::from_reader(reader)?)
    }
}

impl LoggingConfig
{
    pub fn prefix_file(&self, filename: impl AsRef<Path>) -> PathBuf
    {
        let mut path = self.dir.clone();
        path.push(filename);
        path
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

impl From<LogLevel> for LevelFilter
{
    fn from(arg: LogLevel) -> LevelFilter
    {
        match arg
        {
            LogLevel::Trace => LevelFilter::TRACE,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Info  => LevelFilter::INFO,
            LogLevel::Warn  => LevelFilter::WARN,
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Off   => LevelFilter::OFF,
        }
    }
}