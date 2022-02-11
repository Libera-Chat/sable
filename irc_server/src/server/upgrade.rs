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
        })
    }

    /// Restore from a previously saved state.
    ///
    /// The `listener_collection` is only used during the resumption to restore
    /// connection data; the other arguments are as for [`new`](Self::new).
    pub fn restore_from(
            state: ServerState,
            listener_collection: &client_listener::ListenerCollection,
            connection_events: Receiver<ConnectionEvent>,
            rpc_receiver: Receiver<NetworkMessage>,
            to_network: Sender<EventLogUpdate>,
        ) -> std::io::Result<Self>
    {
        let (auth_send, auth_recv) = channel(128);
        let (action_send, action_recv) = unbounded_channel();

        Ok(Self {
            my_id: state.id,
            name: state.name,
            net: state.net,
            epoch: state.epoch,
            id_generator: state.id_generator,
            rpc_receiver,
            event_submitter: to_network,
            action_receiver: action_recv,
            action_submitter: action_send,
            connection_events,
            connections: ConnectionCollection::restore_from(state.connections, listener_collection),
            command_dispatcher: command::CommandDispatcher::new(),
            policy_service: StandardPolicyService::new(),
            auth_client: AuthClient::resume(state.auth_state, auth_send)?,
            auth_events: auth_recv,
            isupport: Self::build_basic_isupport(),
        })
    }
}