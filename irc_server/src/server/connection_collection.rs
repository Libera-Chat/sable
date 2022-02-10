use super::*;
use client_listener::{
    ConnectionId,
    ConnectionData
};
use std::cell::RefCell;

/// Stores the client connections handled by a [`Server`], and allows lookup by
/// either connection ID or user ID
pub(super) struct ConnectionCollection
{
    client_connections: HashMap<ConnectionId, ClientConnection>,
    user_to_connid: HashMap<UserId, ConnectionId>,
}

/// Serialised state of a [`ClientConnection`], for later resumption
#[derive(serde::Serialize,serde::Deserialize)]
pub(super) struct ClientConnectionState
{
    connection_id: ConnectionId,
    connection_data: ConnectionData,
    user_id: Option<UserId>,
    pre_client: Option<PreClient>
}

/// Serialised state of a [`ConnectionCollection`], for later resumption
#[derive(serde::Serialize,serde::Deserialize)]
pub(super) struct ConnectionCollectionState(
    Vec<ClientConnectionState>
);

impl ConnectionCollection
{
    /// Contruct a [`ConnectionCollection`]
    pub fn new() -> Self {
        Self {
            client_connections: HashMap::new(),
            user_to_connid: HashMap::new(),
        }
    }

    /// Insert a new connection, with no associated user ID
    pub fn add(&mut self, id: ConnectionId, conn: ClientConnection)
    {
        self.client_connections.insert(id, conn);
    }

    /// Associate a user ID with an existing connection ID
    pub fn add_user(&mut self, user: UserId, to: ConnectionId)
    {
        self.user_to_connid.insert(user, to);
    }

    /// Remove a connection, by connection ID
    pub fn remove(&mut self, id: ConnectionId)
    {
        if let Some(conn) = self.client_connections.get(&id)
        {
            tracing::trace!("Removing connection {:?}", id);
            if let Some(userid) = conn.user_id {
                self.user_to_connid.remove(&userid);
            }
        }
        self.client_connections.remove(&id);
    }

    /// Remove a connection, by associated user ID
    pub fn remove_user(&mut self, id: UserId) -> Option<ClientConnection>
    {
        if let Some(connid) = self.user_to_connid.remove(&id)
        {
            self.client_connections.remove(&connid)
        }
        else
        {
            None
        }
    }

    /// Look up a connection by ID
    pub fn get(&self, id: ConnectionId) -> Result<&ClientConnection, LookupError>
    {
        self.client_connections.get(&id).ok_or(LookupError::NoSuchConnectionId)
    }

    /// Look up a connection by user ID
    pub fn get_user(&self, id: UserId) -> Result<&ClientConnection, LookupError>
    {
        match self.user_to_connid.get(&id) {
            None => Err(LookupError::NoSuchConnectionId),
            Some(connid) => self.get(*connid)
        }
    }

/*    pub fn get_mut(&mut self, id: ConnectionId) -> Result<&mut ClientConnection, LookupError>
    {
        self.client_connections.get_mut(&id).ok_or(LookupError::NoSuchConnectionId)
    }
*/
    /// Look up a connection by user ID, returning a mutable reference
    pub fn get_user_mut(&mut self, id: UserId) -> Result<&mut ClientConnection, LookupError>
    {
        match self.user_to_connid.get(&id) {
            None => Err(LookupError::NoSuchConnectionId),
            Some(connid) => self.client_connections.get_mut(connid).ok_or(LookupError::NoSuchConnectionId)
        }
    }

    /// Iterate over connections
    pub fn iter(&self) -> impl Iterator<Item=&ClientConnection>
    {
        self.client_connections.values()
    }

    /// Get the number of managed connections
    pub fn len(&self) -> usize
    {
        self.client_connections.len()
    }

    /// Save the collection state for later resumption
    pub fn save_state(self) -> ConnectionCollectionState
    {
        ConnectionCollectionState(
            self.client_connections
                .into_iter()
                .map(|(k,v)| {
                    tracing::trace!("Saving client connection {:?} ({:?})", k, v.user_id);
                    ClientConnectionState {
                        connection_id: k,
                        connection_data: v.connection.save(),
                        user_id: v.user_id,
                        pre_client: v.pre_client.map(|cell| cell.into_inner())
                    }
                })
                .collect()
        )
    }

    /// Restore a collection from a previously stored state
    pub fn restore_from(state: ConnectionCollectionState, listener_collection: &client_listener::ListenerCollection) -> Self
    {
        let mut ret = Self::new();

        for conn_data in state.0.into_iter()
        {
            tracing::trace!("Restoring client connection {:?} ({:?})", conn_data.connection_id, conn_data.user_id);
            let cli_conn = ClientConnection {
                connection: listener_collection.restore_connection(conn_data.connection_data),
                user_id: conn_data.user_id,
                pre_client: conn_data.pre_client.map(|v| RefCell::new(v))
            };
            if let Some(user_id) = &cli_conn.user_id
            {
                ret.user_to_connid.insert(*user_id, conn_data.connection_id);
            }
            ret.client_connections.insert(conn_data.connection_id, cli_conn);
        }

        ret
    }
}