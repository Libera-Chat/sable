use super::*;
use crate::connection_collection::ConnectionCollectionState;
use anyhow::Context;
use async_trait::async_trait;
use client_listener::SavedListenerCollection;
use sable_server::ServerSaveError;

/// Saved state of a [`ClientServer`] for later resumption
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ClientServerState {
    connections: ConnectionCollectionState,
    auth_state: AuthClientState,
    client_caps: CapabilityRepository,
    listener_state: SavedListenerCollection,
}

#[async_trait]
impl sable_server::ServerType for ClientServer {
    type Config = ClientServerConfig;
    type Saved = ClientServerState;

    /// Create a new `ClientServer`
    fn new(
        config: Self::Config,
        tls_data: &TlsData,
        node: Arc<NetworkNode>,
        history_receiver: UnboundedReceiver<NetworkHistoryUpdate>,
    ) -> anyhow::Result<Self> {
        let (action_submitter, action_receiver) = unbounded_channel();
        let (auth_sender, auth_events) = unbounded_channel();
        let (client_send, client_recv) = unbounded_channel();

        let client_listeners = ListenerCollection::new(client_send, config.console_address)
            .context("Could not initialize listener collection")?;

        client_listeners
            .load_tls_certificates(tls_data.key.clone(), tls_data.cert_chain.clone())
            .context("Could not load TLS certificates")?;

        for listener in config.listeners.iter() {
            let conn_type = if listener.tls {
                ConnectionType::Tls
            } else {
                ConnectionType::Clear
            };
            client_listeners
                .add_listener(
                    listener.address.parse().with_context(|| {
                        format!("Invalid listener address: {}", listener.address)
                    })?,
                    conn_type,
                )
                .context("Cannot add listener")?;
        }

        Ok(Self {
            action_receiver: Mutex::new(action_receiver),
            connection_events: Mutex::new(client_recv),
            history_receiver: Mutex::new(history_receiver),
            auth_events: Mutex::new(auth_events),

            auth_client: AuthClient::new(auth_sender)
                .context("Could not initialize auth client")?,

            action_submitter,
            command_dispatcher: CommandDispatcher::new(),
            connections: RwLock::new(ConnectionCollection::new()),
            isupport: Self::build_basic_isupport(),
            client_caps: CapabilityRepository::new(),
            node: node,
            listeners: Movable::new(client_listeners),
        })
    }

    /// Save the server's state for later resumption
    async fn save(mut self) -> Result<ClientServerState, ServerSaveError> {
        Ok(ClientServerState {
            connections: self.connections.into_inner().save_state(),
            auth_state: self
                .auth_client
                .save_state()
                .await
                .map_err(ServerSaveError::IoError)?,
            client_caps: self.client_caps,
            listener_state: self
                .listeners
                .take()
                .unwrap()
                .save()
                .await
                .map_err(ServerSaveError::IoError)?,
        })
    }

    /// Restore from a previously saved state.
    fn restore(
        state: ClientServerState,
        node: Arc<NetworkNode>,
        history_receiver: UnboundedReceiver<NetworkHistoryUpdate>,
    ) -> std::io::Result<Self> {
        let (auth_send, auth_recv) = unbounded_channel();
        let (action_send, action_recv) = unbounded_channel();
        let (client_send, client_recv) = unbounded_channel();

        let listeners = ListenerCollection::resume(state.listener_state, client_send)?;

        Ok(Self {
            node,
            action_receiver: Mutex::new(action_recv),
            action_submitter: action_send,
            connection_events: Mutex::new(client_recv),
            connections: RwLock::new(ConnectionCollection::restore_from(
                state.connections,
                &listeners,
            )),
            command_dispatcher: command::CommandDispatcher::new(),
            auth_client: AuthClient::resume(state.auth_state, auth_send)?,
            auth_events: Mutex::new(auth_recv),
            isupport: Self::build_basic_isupport(),
            client_caps: state.client_caps,
            history_receiver: Mutex::new(history_receiver),
            listeners: Movable::new(listeners),
        })
    }

    async fn run(self: Arc<Self>, shutdown: broadcast::Receiver<ShutdownAction>) {
        self.do_run(shutdown).await;
    }

    async fn shutdown(mut self) {
        if let Some(listeners) = self.listeners.take() {
            listeners.shutdown().await;
        }
    }

    fn handle_remote_command(&self, cmd: RemoteServerRequestType) -> RemoteServerResponse {
        match cmd {
            RemoteServerRequestType::Ping => RemoteServerResponse::Success,
            _ => RemoteServerResponse::NotSupported,
        }
    }
}
