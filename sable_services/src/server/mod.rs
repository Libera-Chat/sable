use crate::{database::{DatabaseConnection, DatabaseError}, model::AccountAuth};
use sable_server::ServerType;
use sable_network::{
    config::TlsData,
    rpc::*,
    node::NetworkNode,
    network::{
        state,
        event::*,
        update::NetworkStateChange,
    },
    id::*,
    modes::*,
};

use std::sync::Arc;

use serde::Deserialize;
use async_trait::async_trait;

use tokio::sync::{
    broadcast,
    mpsc::UnboundedReceiver,
    Mutex,
};

mod sync;
mod command;

#[derive(Deserialize)]
pub struct ServicesConfig
{
    pub database: String,
}

pub struct ServicesServer<DB>
{
    db: DB,
    node: Arc<NetworkNode>,
    history_receiver: Mutex<UnboundedReceiver<sable_network::rpc::NetworkHistoryUpdate>>,
}

#[async_trait]
impl<DB> ServerType for ServicesServer<DB>
    where DB: DatabaseConnection + Send + Sync + 'static
{
    type Config = ServicesConfig;
    type Saved = ();

    fn new(config: Self::Config, _tls_data: &TlsData, node: Arc<NetworkNode>, history_receiver: UnboundedReceiver<sable_network::rpc::NetworkHistoryUpdate>) -> Self
    {
        Self {
            db: DatabaseConnection::connect(config.database).unwrap(),
            node,
            history_receiver: Mutex::new(history_receiver),
        }
    }

    async fn shutdown(self) { }

    async fn run(self: Arc<Self>, mut shutdown_channel: broadcast::Receiver<ShutdownAction>)
    {
        let mut history_receiver = self.history_receiver.lock().await;

        loop {
            tokio::select! {
                _ = shutdown_channel.recv() => { break; }

                update = history_receiver.recv() =>
                {
                    let mut do_burst = false;

                    if let Some(NetworkHistoryUpdate::NewEntry(id)) = update
                    {
                        if let Some(entry) = self.node.history().get(id)
                        {
                            if let NetworkStateChange::NewServer(new_server) = &entry.details
                            {
                                if new_server.server.id == self.node.id()
                                {
                                    // The network has seen us join, so now's the time to sync
                                    // the database and set ourselves as the active services, but
                                    // we need to defer it until after we've dropped the lock guard
                                    // on history.
                                    do_burst = true;
                                }
                            }
                        }
                    }

                    if do_burst
                    {
                        self.burst_to_network().await;
                    }
                }
            }
        }
    }

    async fn save(self) { }

    fn restore(_state: Self::Saved, _node: Arc<NetworkNode>, _history_receiver: UnboundedReceiver<sable_network::rpc::NetworkHistoryUpdate>) -> Self
    {
        unimplemented!("services can't hot-upgrade");
    }

    fn handle_remote_command(&self, req: RemoteServerRequestType) -> RemoteServerResponse
    {
        tracing::debug!(?req, "Got remote request");

        match req
        {
            RemoteServerRequestType::RegisterUser(account_name, password) =>
            {
                tracing::debug!(?account_name, "Got register request");

                self.register_user(account_name, password)
            }
            RemoteServerRequestType::UserLogin(account_id, password) =>
            {
                tracing::debug!(?account_id, "Got login request");

                self.user_login(account_id, password)
            }
            RemoteServerRequestType::RegisterChannel(account_id, channel_id) =>
            {
                tracing::debug!(?account_id, ?channel_id, "Got channel register request");

                self.register_channel(account_id, channel_id)
            }
            RemoteServerRequestType::ModifyAccess { source, id, flags } =>
            {
                tracing::debug!(?source, ?id, ?flags, "Got channel access update");

                self.modify_channel_access(source, id, flags)
            }
            _ =>
            {
                tracing::warn!(?req, "Got unsupported request");
                RemoteServerResponse::NotSupported
            }
        }
    }
}