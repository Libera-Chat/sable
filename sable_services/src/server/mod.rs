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

                let new_account_id = self.node.ids().next_account();

                let Ok(password_hash) = bcrypt::hash(password, bcrypt::DEFAULT_COST) else {
                    tracing::error!(?account_name, "Failed to hash password for new account");

                    return RemoteServerResponse::Error("Failed to hash password".to_string());
                };

                let account_data = state::Account {
                    id: new_account_id,
                    name: account_name,
                };
                let auth_data = AccountAuth {
                    account: new_account_id,
                    password_hash
                };

                match self.db.new_account(account_data, auth_data)
                {
                    Ok(new_account) =>
                    {
                        tracing::debug!(?new_account, "Successfully created account");
                        let id = new_account.id;
                        self.node.submit_event(id, AccountUpdate { data: Some(new_account) });
                        RemoteServerResponse::LogUserIn(id)
                    }
                    Err(DatabaseError::DuplicateId | DatabaseError::DuplicateName) =>
                    {
                        tracing::debug!(?account_name, "Duplicate account name/id");
                        RemoteServerResponse::AlreadyExists
                    }
                    Err(error) =>
                    {
                        tracing::error!(?error, "Error creating account");
                        RemoteServerResponse::Error("Unknown error".to_string())
                    }
                }
            }
            RemoteServerRequestType::UserLogin(account_id, password) =>
            {
                tracing::debug!(?account_id, "Got login request");

                let Ok(auth) = self.db.auth_for_account(account_id) else {
                    tracing::error!(?account_id, "Error looking up account");
                    return RemoteServerResponse::Error("Couldn't look up account".to_string());
                };

                match bcrypt::verify(password, &auth.password_hash)
                {
                    Ok(true) => {
                        tracing::debug!("login successful");
                        RemoteServerResponse::LogUserIn(account_id)
                    }
                    Ok(false) => {
                        tracing::debug!("wrong password");
                        RemoteServerResponse::InvalidCredentials
                    }
                    Err(_) => RemoteServerResponse::Error("Couldn't verify password".to_string())
                }
            }
            RemoteServerRequestType::RegisterChannel(account_id, channel_id) =>
            {
                tracing::debug!(?account_id, ?channel_id, "Got channel register request");

                let net = self.node.network();

                let Ok(channel) = net.channel(channel_id) else {
                    return RemoteServerResponse::Error("Channel doesn't exist".to_string());
                };

                let new_channel_registration = state::ChannelRegistration {
                    id: self.node.ids().next_channel_registration(),
                    channelname: channel.name().clone()
                };

                let new_channel_access = state::ChannelAccess {
                    id: ChannelAccessId::new(account_id, new_channel_registration.id),
                    flags: ChannelAccessFlag::Founder | ChannelAccessFlag::Access | ChannelAccessFlag::Op
                };

                match self.db.new_channel_registration(new_channel_registration, new_channel_access)
                {
                    Ok((channel_registration, channel_access)) =>
                    {
                        self.node.submit_event(channel_registration.id, ChannelRegistrationUpdate { data: Some(channel_registration) });
                        self.node.submit_event(channel_access.id, ChannelAccessUpdate { data: Some(channel_access) });
                        RemoteServerResponse::Success
                    }
                    Err(DatabaseError::DuplicateName) =>
                    {
                        RemoteServerResponse::AlreadyExists
                    }
                    Err(error) =>
                    {
                        let channel_name = channel.name();
                        tracing::error!(?error, ?channel_name, "Unexpected error registering channel");
                        RemoteServerResponse::Error("Unexpected error".to_string())
                    }
                }
            }
            _ =>
            {
                tracing::warn!(?req, "Got unsupported request");
                RemoteServerResponse::NotSupported
            }
        }
    }
}