use sable_network::{config::TlsData, node::*, rpc::*};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc::UnboundedReceiver};

#[derive(Debug, Error)]
pub enum ServerSaveError {
    #[error("{0}")]
    IoError(std::io::Error),
    #[error("Unknown error: {0}")]
    EventLogSaveError(sable_network::sync::EventLogSaveError),
}

/// Trait to be implemented by providers of server application logic.
///
/// An implementor of this trait can be constructed and used by [`run_server`](crate::run::run_server).
#[async_trait]
pub trait ServerType: Send + Sync + Sized + 'static {
    /// The configuration settings required for this server type. A field named "server" of this
    /// type must be read from the server's config file.
    type Config: DeserializeOwned;

    /// The configuration settings after validation or pre-processing. This could include reading
    /// the content of a file referenced in the `Config` type.
    type ProcessedConfig;

    /// An error type returned if config validation fails
    type ConfigError: std::error::Error + Send + Sync;

    /// A type describing the saved state of this server type, to be resumed after a code upgrade
    type Saved: Serialize + DeserializeOwned;

    /// Validate a `Config` and transform it into a `ProcessedConfig`
    fn validate_config(config: &Self::Config) -> Result<Self::ProcessedConfig, Self::ConfigError>;

    /// Construct a new server
    fn new(
        config: Self::ProcessedConfig,
        tls_data: &TlsData,
        node: Arc<NetworkNode>,
        history_receiver: UnboundedReceiver<NetworkHistoryUpdate>,
    ) -> anyhow::Result<Self>;

    /// Run the application logic. `shutdown_channel` will be signalled with an `ShutdownAction` when
    /// the server should be stopped.
    async fn run(self: Arc<Self>, shutdown_channel: broadcast::Receiver<ShutdownAction>);

    /// Perform any actions required to shut down the server, if it will not be resumed
    async fn shutdown(self);

    /// Save state for later resumption
    async fn save(self) -> Result<Self::Saved, ServerSaveError>;

    /// Restore from saved state
    fn restore(
        state: Self::Saved,
        node: Arc<NetworkNode>,
        history_receiver: UnboundedReceiver<NetworkHistoryUpdate>,
        config: &Self::ProcessedConfig,
    ) -> std::io::Result<Self>;

    /// Handle a request originating from a remote server
    fn handle_remote_command(&self, request: RemoteServerRequestType) -> RemoteServerResponse;
}
