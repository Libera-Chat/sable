use super::*;
use connection_collection::ConnectionCollectionState;
use auth_client::{
    AuthClient,
    AuthClientState
};
use sable_network::server::{
    Server,
    ServerState,
};

/// Saved state of a [`Server`] for later resumption
#[derive(serde::Serialize,serde::Deserialize)]
pub struct ClientServerState
{
    server_state: ServerState,
    connections: ConnectionCollectionState,
    auth_state: AuthClientState,
    client_caps: CapabilityRepository,
}

impl ClientServer
{
    /// Save the server's state for later resumption
    pub async fn save_state(self) -> std::io::Result<ClientServerState>
    {
        Ok(ClientServerState {
            server_state: Arc::try_unwrap(self.server).map_err(|_| std::io::ErrorKind::Other)?.save_state(),
            connections: self.connections.save_state(),
            auth_state: self.auth_client.save_state().await?,
            client_caps: self.client_caps,
        })
    }

    /// Restore from a previously saved state.
    ///
    /// The `listener_collection` is only used during the resumption to restore
    /// connection data; the other arguments are as for [`new`](Self::new).
    pub fn restore_from(
            state: ClientServerState,
            event_log: Arc<ReplicatedEventLog>,
            listener_collection: &client_listener::ListenerCollection,
            connection_events: UnboundedReceiver<ConnectionEvent>,
            rpc_receiver: UnboundedReceiver<NetworkMessage>,
        ) -> std::io::Result<Self>
    {
        let (auth_send, auth_recv) = unbounded_channel();
        let (action_send, action_recv) = unbounded_channel();
        let (state_change_sender, state_change_receiver) = unbounded_channel();

        Ok(Self {
            server: Arc::new(Server::restore_from(state.server_state, event_log, rpc_receiver, state_change_sender)?),
            action_receiver: action_recv,
            action_submitter: action_send.clone(),
            connection_events,
            connections: ConnectionCollection::restore_from(state.connections, listener_collection, action_send),
            command_dispatcher: command::CommandDispatcher::new(),
            policy_service: StandardPolicyService::new(),
            auth_client: AuthClient::resume(state.auth_state, auth_send)?,
            auth_events: auth_recv,
            isupport: Self::build_basic_isupport(),
            client_caps: state.client_caps,
            state_change_receiver
        })
    }
}