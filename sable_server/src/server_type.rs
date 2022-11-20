use sable_network::{
    config::TlsData,
    node::*,
    rpc::{NetworkHistoryUpdate, ShutdownAction},
};

use tokio::sync::{
    mpsc::UnboundedReceiver,
    broadcast
};
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize,de::DeserializeOwned};

/// Trait to be implemented by providers of server application logic.
///
/// An implementor of this trait can be constructed and used by [`run_server`](crate::run::run_server).
#[async_trait]
pub trait ServerType : Send + Sync + 'static
{
    /// The configuration settings required for this server type
    type Config: DeserializeOwned;

    /// A type describing the saved state of this server type, to be resumed after a code upgrade
    type Saved: Serialize + DeserializeOwned;

    /// Construct a new server
    fn new(config: Self::Config, tls_data: &TlsData, node: Arc<NetworkNode>, history_receiver: UnboundedReceiver<NetworkHistoryUpdate>) -> Self;

    /// Run the application logic. `shutdown_channel` will be signalled with an `ShutdownAction` when
    /// the server should be stopped.
    async fn run(&self, shutdown_channel: broadcast::Receiver<ShutdownAction>);

    /// Perform any actions required to shut down the server, if it will not be resumed
    async fn shutdown(self);

    /// Save state for later resumption
    async fn save(self) -> Self::Saved;

    /// Restore from saved state
    fn restore(stats: Self::Saved, node: Arc<NetworkNode>, history_receiver: UnboundedReceiver<NetworkHistoryUpdate>) -> Self;
}