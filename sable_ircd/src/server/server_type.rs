use sable_network::node::NetworkNodeState;
use client_listener::SavedListenerCollection;
use crate::connection_collection::ConnectionCollectionState;
use async_trait::async_trait;
use super::*;

/// Saved state of a [`ClientServer`] for later resumption
#[derive(serde::Serialize,serde::Deserialize)]
pub struct ClientServerState
{
    node_state: NetworkNodeState,
    connections: ConnectionCollectionState,
    auth_state: AuthClientState,
    client_caps: CapabilityRepository,
    listener_state: SavedListenerCollection,
}

#[async_trait]
impl sable_server::ServerType for ClientServer
{
    type Config = ClientServerConfig;
    type Saved = ClientServerState;

    /// Create a new `ClientServer`
    fn new(config: Self::Config,
           tls_data: &TlsData,
            node: Arc<NetworkNode>,
           history_receiver: UnboundedReceiver<NetworkHistoryUpdate>,
        ) -> Self
    {
        let (action_submitter, action_receiver) = unbounded_channel();
        let (auth_sender, auth_events) = unbounded_channel();
        let (client_send, client_recv) = unbounded_channel();

        let client_listeners = ListenerCollection::new(client_send).unwrap();

        client_listeners.load_tls_certificates(tls_data.key.clone(), tls_data.cert_chain.clone()).unwrap();

        for listener in config.listeners.iter()
        {
            let conn_type = if listener.tls {ConnectionType::Tls} else {ConnectionType::Clear};
            client_listeners.add_listener(listener.address.parse().unwrap(), conn_type).unwrap();
        }

        Self {
            action_receiver: Mutex::new(action_receiver),
            connection_events: Mutex::new(client_recv),
            history_receiver: Mutex::new(history_receiver),
            auth_events: Mutex::new(auth_events),

            auth_client: AuthClient::new(auth_sender).unwrap(),

            action_submitter,
            command_dispatcher: CommandDispatcher::new(),
            connections: RwLock::new(ConnectionCollection::new()),
            isupport: Self::build_basic_isupport(),
            client_caps: CapabilityRepository::new(),
            server: node,
            listeners: Movable::new(client_listeners),
        }
    }

    /// Save the server's state for later resumption
    async fn save(mut self) -> ClientServerState
    {
        ClientServerState {
            node_state: Arc::try_unwrap(self.server)
                                .unwrap_or_else(|_| panic!("failed to unwrap node"))
                                .save_state(),
            connections: self.connections.into_inner().save_state(),
            auth_state: self.auth_client.save_state().await.unwrap(),
            client_caps: self.client_caps,
            listener_state: self.listeners.take().unwrap().save().await.expect("failed to save listener state"),
        }
    }

    /// Restore from a previously saved state.
    ///
    /// The `listener_collection` is only used during the resumption to restore
    /// connection data; the other arguments are as for [`new`](Self::new).
    fn restore(
            state: ClientServerState,
            server: Arc<NetworkNode>,
            history_receiver: UnboundedReceiver<NetworkHistoryUpdate>,
        ) -> Self
    {
        let (auth_send, auth_recv) = unbounded_channel();
        let (action_send, action_recv) = unbounded_channel();
        let (client_send, client_recv) = unbounded_channel();

        let listeners = ListenerCollection::resume(state.listener_state, client_send)
                            .expect("failed to restore listener collection");

        Self {
            server,
            action_receiver: Mutex::new(action_recv),
            action_submitter: action_send,
            connection_events: Mutex::new(client_recv),
            connections: RwLock::new(ConnectionCollection::restore_from(state.connections, &listeners)),
            command_dispatcher: command::CommandDispatcher::new(),
            auth_client: AuthClient::resume(state.auth_state, auth_send).expect("Failed to reload auth client"),
            auth_events: Mutex::new(auth_recv),
            isupport: Self::build_basic_isupport(),
            client_caps: state.client_caps,
            history_receiver: Mutex::new(history_receiver),
            listeners: Movable::new(listeners),
        }
    }

    async fn run(&self, shutdown: broadcast::Receiver<ShutdownAction>)
    {
        self.do_run(shutdown).await;
    }

    async fn shutdown(mut self)
    {
        if let Some(listeners) = self.listeners.take()
        {
            listeners.shutdown().await;
        }
    }
}