use std::convert::Infallible;

use sable_network::prelude::*;
use sable_server::ServerType;
use serde::Deserialize;
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};

use std::sync::Arc;

mod update_handler;

#[derive(Debug, Clone, Deserialize)]
pub struct HistoryServerConfig {
    pub database: String,
}

pub struct HistoryServer {
    node: Arc<NetworkNode>,
    history_receiver: Mutex<UnboundedReceiver<sable_network::rpc::NetworkHistoryUpdate>>,
}

impl ServerType for HistoryServer {
    type Config = HistoryServerConfig;

    type ProcessedConfig = HistoryServerConfig;

    type ConfigError = Infallible;

    type Saved = ();

    fn validate_config(config: &Self::Config) -> Result<Self::ProcessedConfig, Self::ConfigError> {
        Ok(config.clone())
    }

    async fn new(
        _config: Self::ProcessedConfig,
        _tls_data: &sable_network::config::TlsData,
        node: std::sync::Arc<sable_network::prelude::NetworkNode>,
        history_receiver: tokio::sync::mpsc::UnboundedReceiver<
            sable_network::rpc::NetworkHistoryUpdate,
        >,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            node,
            history_receiver: Mutex::new(history_receiver),
        })
    }

    async fn run(
        self: std::sync::Arc<Self>,
        mut shutdown_channel: tokio::sync::broadcast::Receiver<sable_network::rpc::ShutdownAction>,
    ) {
        let mut history_receiver = self.history_receiver.lock().await;

        loop {
            tokio::select! {
                _ = shutdown_channel.recv() => { break; }

                update = history_receiver.recv() =>
                {
                    let Some(update) = update else { break; };

                    self.handle_history_update(update);
                }
            }
        }
    }

    async fn shutdown(self) {}

    async fn save(self) -> Result<Self::Saved, sable_server::ServerSaveError> {
        Ok(())
    }

    fn restore(
        _state: Self::Saved,
        _node: std::sync::Arc<sable_network::prelude::NetworkNode>,
        _history_receiver: tokio::sync::mpsc::UnboundedReceiver<
            sable_network::rpc::NetworkHistoryUpdate,
        >,
        _config: &Self::ProcessedConfig,
    ) -> std::io::Result<Self> {
        unimplemented!("history servers can't hot-upgrade");
    }

    fn handle_remote_command(
        &self,
        _request: sable_network::rpc::RemoteServerRequestType,
    ) -> sable_network::rpc::RemoteServerResponse {
        todo!()
    }
}
