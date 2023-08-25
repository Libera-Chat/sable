use crate::{
    database::{DatabaseConnection, DatabaseError},
    model::*,
};
use command::CommandError;

use sable_network::{
    config::TlsData,
    id::*,
    network::{
        event::*, state, state::ChannelAccessFlag, state::ChannelRoleName,
        update::NetworkStateChange,
    },
    node::NetworkNode,
    prelude::LookupError,
    rpc::*,
};
use sable_server::ServerType;

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use serde::Deserialize;

use tokio::sync::{broadcast, mpsc::UnboundedReceiver, Mutex};

use dashmap::DashMap;

mod command;
mod roles;
mod sasl;
mod sync;

#[derive(Deserialize)]
pub struct ServicesConfig {
    pub database: String,
    pub default_roles: HashMap<ChannelRoleName, Vec<ChannelAccessFlag>>,
}

pub struct ServicesServer<DB> {
    db: DB,
    node: Arc<NetworkNode>,
    history_receiver: Mutex<UnboundedReceiver<sable_network::rpc::NetworkHistoryUpdate>>,
    config: ServicesConfig,
    sasl_sessions: DashMap<SaslSessionId, SaslSession>,
    sasl_mechanisms: HashMap<String, Box<dyn sasl::SaslMechanism<DB>>>,
}

#[async_trait]
impl<DB> ServerType for ServicesServer<DB>
where
    DB: DatabaseConnection + Send + Sync + 'static,
{
    type Config = ServicesConfig;
    type Saved = ();

    fn new(
        config: Self::Config,
        _tls_data: &TlsData,
        node: Arc<NetworkNode>,
        history_receiver: UnboundedReceiver<sable_network::rpc::NetworkHistoryUpdate>,
    ) -> Self {
        if !config
            .default_roles
            .contains_key(&ChannelRoleName::BuiltinOp)
            || !config
                .default_roles
                .contains_key(&ChannelRoleName::BuiltinVoice)
            || !config
                .default_roles
                .contains_key(&ChannelRoleName::BuiltinFounder)
        {
            tracing::error!(
                "Services configuration doesn't define builtin op/voice or founder roles; aborting"
            );
            panic!("Builtin roles not defined");
        }

        Self {
            db: DatabaseConnection::connect(&config.database).unwrap(),
            node,
            history_receiver: Mutex::new(history_receiver),
            config,
            sasl_sessions: DashMap::new(),
            sasl_mechanisms: sasl::build_mechanisms(),
        }
    }

    async fn shutdown(self) {}

    async fn run(self: Arc<Self>, mut shutdown_channel: broadcast::Receiver<ShutdownAction>) {
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

    async fn save(self) {}

    fn restore(
        _state: Self::Saved,
        _node: Arc<NetworkNode>,
        _history_receiver: UnboundedReceiver<sable_network::rpc::NetworkHistoryUpdate>,
    ) -> Self {
        unimplemented!("services can't hot-upgrade");
    }

    fn handle_remote_command(&self, req: RemoteServerRequestType) -> RemoteServerResponse {
        tracing::debug!(?req, "Got remote request");

        use RemoteServerRequestType::*;

        let result = match req {
            RegisterUser(account_name, password) => {
                tracing::debug!(?account_name, "Got register request");

                self.register_user(account_name, password)
            }
            UserLogin(account_id, password) => {
                tracing::debug!(?account_id, "Got login request");

                self.user_login(account_id, password)
            }
            RegisterChannel(account_id, channel_id) => {
                tracing::debug!(?account_id, ?channel_id, "Got channel register request");

                self.register_channel(account_id, channel_id)
            }
            ModifyAccess { source, id, role } => {
                tracing::debug!(?source, ?id, ?role, "Got channel access update");

                self.modify_channel_access(source, id, role)
            }
            CreateRole {
                source,
                channel,
                name,
                flags,
            } => {
                tracing::debug!(?source, ?channel, ?name, ?flags, "Got role creation");

                self.create_role(source, channel, name, flags)
            }
            ModifyRole { source, id, flags } => {
                tracing::debug!(?source, ?id, ?flags, "Got modify role");

                self.modify_role(source, id, flags)
            }
            BeginAuthenticate(session, mechanism) => {
                tracing::debug!(?session, ?mechanism, "Got begin authenticate");

                self.begin_authenticate(session, mechanism)
            }
            Authenticate(session, data) => {
                tracing::debug!(?session, ?data, "Got authenticate data");

                self.authenticate(session, data)
            }
            AbortAuthenticate(session) => {
                tracing::debug!(?session, "Got abort authenticate");

                self.abort_authenticate(session)
            }
            AddAccountFingerprint(acc, fp) => {
                tracing::debug!(?acc, ?fp, "Got add fingerprint");

                self.user_add_fp(acc, fp)
            }
            RemoveAccountFingerprint(acc, fp) => {
                tracing::debug!(?acc, ?fp, "Got remove fingerprint");

                self.user_del_fp(acc, fp)
            }
            Ping => {
                tracing::warn!(?req, "Got unsupported request");
                Ok(RemoteServerResponse::NotSupported)
            }
        };

        match result {
            Ok(response) => response,
            Err(CommandError::LookupError(
                LookupError::NoSuchAccount(_) | LookupError::NoSuchAccountNamed(_),
            )) => RemoteServerResponse::NoAccount,
            Err(CommandError::LookupError(
                LookupError::NoSuchChannelRegistration(_) | LookupError::ChannelNotRegistered(_),
            )) => RemoteServerResponse::ChannelNotRegistered,
            Err(e) => RemoteServerResponse::Error(e.to_string()),
        }
    }
}
