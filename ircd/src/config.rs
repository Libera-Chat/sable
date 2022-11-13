use crate::*;
use crate::strip_comments::StripComments;
use std::net::SocketAddr;
use std::collections::HashMap;
use tracing_core::{
    LevelFilter,
};
use sable_ircd::server::config::*;
use sable_network::config::*;

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

    pub server: ClientServerConfig,

    pub management: ManagementConfig,

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

impl ManagementConfig
{
    pub fn load_client_ca(&self) -> std::io::Result<Vec<u8>>
    {
        let ca_file = File::open(&self.client_ca)?;
        let mut ca_reader = BufReader::new(ca_file);
        Ok(rustls_pemfile::certs(&mut ca_reader)?.remove(0))
    }
}

pub fn load_network_config(filename: impl AsRef<Path>) -> Result<sable_network::network::config::NetworkConfig, sable_network::sync::ConfigError>
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