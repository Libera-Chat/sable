use super::*;

use sable_network::prelude::*;
use client_listener::{
    ConnectionId,
};
use std::{
    collections::HashMap,
};

use tokio::sync::mpsc::{
    UnboundedSender,
};

/// Stores the client connections handled by a [`Server`], and allows lookup by
/// either connection ID or user ID
pub(super) struct ConnectionCollection
{
    client_connections: HashMap<ConnectionId, ClientConnection>,
    user_to_connid: HashMap<UserId, ConnectionId>,
    action_sender: UnboundedSender<CommandAction>,
}

/// Serialised state of a [`ConnectionCollection`], for later resumption
#[derive(serde::Serialize,serde::Deserialize)]
pub(super) struct ConnectionCollectionState(
    Vec<(ConnectionId,ClientConnectionState)>
);

impl ConnectionCollection
{
    /// Contruct a [`ConnectionCollection`]
    pub fn new(action_sender: UnboundedSender<CommandAction>) -> Self {
        Self {
            client_connections: HashMap::new(),
            user_to_connid: HashMap::new(),
            action_sender
        }
    }

    /// Called by the [`Server`] for each new network message received.
    ///
    /// Finds the relevant [`ClientConnection`] and adds to the receive queue.
    /// If the queue is full, the connection is closed with an appropriate error message
    pub fn new_message(&mut self, conn_id: ConnectionId, message: String)
    {
        if let Some(conn) = self.client_connections.get_mut(&conn_id)
        {
            if conn.new_message(message).is_err()
            {
                // An error return here means that the connection's receive queue is full,
                // so they should be disconnected for flooding. First, check whether it's a
                // registered user connection, or a pre-client
                if let Some(user_id) = conn.user_id
                {
                    // Registered user, so we need to inform the network they're going
                    let event_detail = event::details::UserQuit { message: "Excess Flood".to_string() };
                    self.action_sender.send(CommandAction::StateChange(user_id.into(), event_detail.into())).expect("Failed to submit event");
                }
                self.remove(conn_id);
            }
        }
    }

    pub fn poll_messages(&mut self) -> impl Iterator<Item=(ConnectionId, String)> + '_
    {
        self.client_connections.iter_mut().flat_map(|(id,conn)| conn.poll_messages().map(move |message| (*id, message)))
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

    /// Look up a connection by ID, returning a mutable reference
    pub fn get_mut(&mut self, id: ConnectionId) -> Result<&mut ClientConnection, LookupError>
    {
        self.client_connections.get_mut(&id).ok_or(LookupError::NoSuchConnectionId)
    }

    /// Look up a connection by user ID
    pub fn get_user(&self, id: UserId) -> Result<&ClientConnection, LookupError>
    {
        match self.user_to_connid.get(&id) {
            None => Err(LookupError::NoSuchConnectionId),
            Some(connid) => self.get(*connid)
        }
    }

    /// Look up a connection by user ID, returning a mutable reference
    pub fn get_user_mut(&mut self, id: UserId) -> Result<&mut ClientConnection, LookupError>
    {
        match self.user_to_connid.get(&id) {
            None => Err(LookupError::NoSuchConnectionId),
            Some(connid) => self.client_connections.get_mut(connid).ok_or(LookupError::NoSuchConnectionId)
        }
    }
/*
    /// Iterate over connections
    pub fn iter(&self) -> impl Iterator<Item=&ClientConnection>
    {
        self.client_connections.values()
    }
*/
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
                    (k, v.save())
                })
                .collect()
        )
    }

    /// Restore a collection from a previously stored state
    pub fn restore_from(state: ConnectionCollectionState,
                        listener_collection: &client_listener::ListenerCollection,
                        action_sender: UnboundedSender<CommandAction>) -> Self
    {
        let mut ret = Self::new(action_sender);

        for (conn_id, conn_data) in state.0.into_iter()
        {
            let cli_conn = ClientConnection::restore(conn_data, listener_collection);
            if let Some(user_id) = &cli_conn.user_id
            {
                ret.user_to_connid.insert(*user_id, conn_id);
            }
            ret.client_connections.insert(conn_id, cli_conn);
        }

        ret
    }
}