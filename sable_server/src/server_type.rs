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

#[async_trait]
pub trait ServerType : Send + Sync + 'static
{
    type Config: DeserializeOwned;
    type Saved: Serialize + DeserializeOwned;

    fn new(config: Self::Config, tls_data: &TlsData, node: Arc<NetworkNode>, history_receiver: UnboundedReceiver<NetworkHistoryUpdate>) -> Self;

    async fn run(&self, shutdown_channel: broadcast::Receiver<ShutdownAction>);

    async fn shutdown(self);

    async fn save(self) -> Self::Saved;
    fn restore(stats: Self::Saved, node: Arc<NetworkNode>, history_receiver: UnboundedReceiver<NetworkHistoryUpdate>) -> Self;
}