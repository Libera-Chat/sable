use std::collections::HashMap;
use std::net::SocketAddr;
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};
use tracing_core::LevelFilter;

/// A client certificate fingerprint which is authorised for the management interface
#[derive(Clone, Debug, serde::Deserialize)]
pub struct AuthorisedFingerprint {
    pub name: String,
    pub fingerprint: String,
}

/// One of the special built-in log targets
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuiltinLogTarget {
    Stdout,
    Stderr,
}

/// A log target
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum LogTarget {
    File { filename: PathBuf },
    Builtin(BuiltinLogTarget),
}

/// Log levels. Equivalent to those defined in `tracing`
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Off,
}

/// A log configuration entry. Defines a log target along with the messages to be sent to it.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct LogEntry {
    pub target: LogTarget,
    pub category: Option<String>,
    #[serde(default)]
    pub modules: Vec<String>,
    pub level: Option<LogLevel>,
}

/// Configuration of the logging system
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LoggingConfig {
    /// Top level directory. All other paths are relative to this
    pub dir: PathBuf,
    /// File to which stdout will be redirected, if daemonised
    pub stdout: Option<PathBuf>,
    /// File to which stderr will be redirected, if daemonised
    pub stderr: Option<PathBuf>,
    /// File in which to store the server PID, if daemonised
    pub pidfile: Option<PathBuf>,
    /// Minimum level to be logged, if not overridden per target or per module
    pub default_level: Option<LogLevel>,
    /// Per-module settings for minimum log level
    pub module_levels: HashMap<String, LogLevel>,
    /// Log targets
    pub targets: Vec<LogEntry>,
    /// Optional listener address for the tokio console
    pub console_address: Option<std::net::SocketAddr>,
}

/// Configuration of the management service
#[derive(Clone, Debug, serde::Deserialize)]
pub struct ManagementConfig {
    /// Listener address
    pub address: SocketAddr,
    /// Certificate authority used to authenticate clients
    pub client_ca: PathBuf,
    /// List of client certificate fingerprints authorised to connect
    pub authorised_fingerprints: Vec<AuthorisedFingerprint>,
}

impl LoggingConfig {
    pub fn prefix_file(&self, filename: impl AsRef<Path>) -> PathBuf {
        let mut path = self.dir.clone();
        path.push(filename);
        path
    }
}

impl ManagementConfig {
    /// Load the client CA from the path specified in this configuration
    pub fn load_client_ca(&self) -> std::io::Result<Vec<u8>> {
        let ca_file = File::open(&self.client_ca)?;
        let mut ca_reader = BufReader::new(ca_file);
        Ok(rustls_pemfile::certs(&mut ca_reader)?.remove(0))
    }
}

/// Load a network state configuration for boostrapping
pub fn load_network_config(
    filename: impl AsRef<Path>,
) -> Result<sable_network::network::config::NetworkConfig, sable_network::sync::ConfigError> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

impl From<LogLevel> for LevelFilter {
    fn from(arg: LogLevel) -> LevelFilter {
        match arg {
            LogLevel::Trace => LevelFilter::TRACE,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Off => LevelFilter::OFF,
        }
    }
}
