use super::*;
use connection_collection::ConnectionCollectionState;
use auth_client::{
    AuthClient,
    AuthClientState
};
use sable_network::node::{
    NetworkNode,
    NetworkNodeState,
};

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

impl ClientServer
{
    /// Save the server's state for later resumption
    pub async fn save_state(mut self) -> std::io::Result<ClientServerState>
    {
        Ok(ClientServerState {
            node_state: Arc::try_unwrap(self.server).map_err(|_| std::io::ErrorKind::Other)?.save_state(),
            connections: self.connections.into_inner().save_state(),
            auth_state: self.auth_client.save_state().await?,
            client_caps: self.client_caps,
            listener_state: self.listeners.take().unwrap().save().await?,
        })
    }

    /// Restore from a previously saved state.
    ///
    /// The `listener_collection` is only used during the resumption to restore
    /// connection data; the other arguments are as for [`new`](Self::new).
    pub fn restore_from(
            state: ClientServerState,
            server: Arc<NetworkNode>,
            history_receiver: UnboundedReceiver<NetworkHistoryUpdate>,
        ) -> std::io::Result<Self>
    {
        let (auth_send, auth_recv) = unbounded_channel();
        let (action_send, action_recv) = unbounded_channel();
        let (client_send, client_recv) = unbounded_channel();

        let listeners = ListenerCollection::resume(state.listener_state, client_send)?;

        Ok(Self {
            server,
            action_receiver: Mutex::new(action_recv),
            action_submitter: action_send,
            connection_events: Mutex::new(client_recv),
            connections: RwLock::new(ConnectionCollection::restore_from(state.connections, &listeners)),
            command_dispatcher: command::CommandDispatcher::new(),
            auth_client: AuthClient::resume(state.auth_state, auth_send)?,
            auth_events: Mutex::new(auth_recv),
            isupport: Self::build_basic_isupport(),
            client_caps: state.client_caps,
            history_receiver: Mutex::new(history_receiver),
            listeners: Movable::new(listeners),
        })
    }
}