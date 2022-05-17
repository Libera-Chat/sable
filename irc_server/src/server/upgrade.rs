use super::*;
use connection_collection::ConnectionCollectionState;
use auth_client::{
    AuthClient,
    AuthClientState
};

/// Saved state of a [`Server`] for later resumption
#[derive(serde::Serialize,serde::Deserialize)]
pub struct ServerState
{
    id: ServerId,
    name: ServerName,
    net: Network,
    epoch: EpochId,
    id_generator: ObjectIdGenerator,
    connections: ConnectionCollectionState,
    auth_state: AuthClientState,
    client_caps: CapabilityRepository,
}

impl Server
{
    /// Save the server's state for later resumption
    pub async fn save_state(self) -> std::io::Result<ServerState>
    {
        Ok(ServerState {
            id: self.my_id,
            name: self.name,
            net: self.net,
            epoch: self.epoch,
            id_generator: self.id_generator,
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
            state: ServerState,
            event_log: Arc<ReplicatedEventLog>,
            listener_collection: &client_listener::ListenerCollection,
            connection_events: UnboundedReceiver<ConnectionEvent>,
            rpc_receiver: UnboundedReceiver<NetworkMessage>,
        ) -> std::io::Result<Self>
    {
        let (auth_send, auth_recv) = unbounded_channel();
        let (action_send, action_recv) = unbounded_channel();

        Ok(Self {
            my_id: state.id,
            name: state.name,
            version: Self::build_version(),
            net: state.net,
            epoch: state.epoch,
            id_generator: state.id_generator,
            rpc_receiver,
            event_log,
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
        })
    }
}