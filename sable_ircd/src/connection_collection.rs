use super::*;

use sable_network::prelude::*;
use client_listener::{
    ConnectionId,
};
use std::{
    collections::HashMap,
};

/// Stores the client connections handled by a [`ClientServer`], and allows lookup by
/// either connection ID or user ID
pub(super) struct ConnectionCollection
{
    client_connections: HashMap<ConnectionId, ClientConnection>,
    user_to_connid: HashMap<UserId, Vec<ConnectionId>>,
    flooded_connections: Vec<ClientConnection>,
}

/// Iterator over connections belonging to a given user
pub(super) struct UserConnectionIter<'a>
{
    connections: &'a HashMap<ConnectionId, ClientConnection>,
    iter: Option<std::slice::Iter<'a, ConnectionId>>,
}

/// Mutable iterator over connections belonging to a given user
pub(super) struct UserConnectionIterMut<'a>
{
    connections: &'a mut HashMap<ConnectionId, ClientConnection>,
    iter: Option<std::slice::Iter<'a, ConnectionId>>,
}

/// Serialised state of a [`ConnectionCollection`], for later resumption
#[derive(serde::Serialize,serde::Deserialize)]
pub(super) struct ConnectionCollectionState{
    clients: Vec<(ConnectionId,ClientConnectionState)>,
    flooded: Vec<ClientConnectionState>
}

impl ConnectionCollection
{
    /// Contruct a [`ConnectionCollection`]
    pub fn new() -> Self {
        Self {
            client_connections: HashMap::new(),
            user_to_connid: HashMap::new(),
            flooded_connections: Vec::new(),
        }
    }

    /// Called by the [`ClientServer`] for each new network message received.
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
                if let Some(conn) = self.client_connections.remove(&conn_id)
                {
                    self.flooded_connections.push(conn);
                }
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
        let user_conns = self.user_to_connid.entry(user).or_default();

        // Important: this vec cannot contain duplicate IDs; not only will it
        // result in sending messages twice to the same connection, it may result
        // in memory unsafety when iterating mutably over the user's connections
        //
        // See the safety comment in [`UserConnectionIterMut::next`]
        if ! user_conns.contains(&to)
        {
            user_conns.push(to);
        }
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

    /// Remove all connections associated with the given user ID
    pub fn remove_user(&mut self, id: UserId)
    {
        if let Some(conn_ids) = self.user_to_connid.remove(&id)
        {
            for connid in conn_ids
            {
                self.client_connections.remove(&connid);
            }
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

    /// Iterate over all connections associated with the given user
    pub fn get_user(&self, id: UserId) -> UserConnectionIter<'_>
    {
        UserConnectionIter {
            connections: &self.client_connections,
            iter: self.user_to_connid.get(&id).map(|c| c.iter())
        }
    }

    /// Iterate mutably over all connections associated with the given user
    pub fn get_user_mut(&mut self, id: UserId) -> UserConnectionIterMut<'_>
    {
        UserConnectionIterMut {
            connections: &mut self.client_connections,
            iter: self.user_to_connid.get(&id).map(|c| c.iter())
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

    /// Drain the list of flooded-off connections for processing
    pub fn flooded_connections(&mut self) -> impl Iterator<Item=ClientConnection> + '_
    {
        self.flooded_connections.drain(..)
    }

    /// Save the collection state for later resumption
    pub fn save_state(self) -> ConnectionCollectionState
    {
        ConnectionCollectionState{
            clients: self.client_connections
                .into_iter()
                .map(|(k,v)| {
                    tracing::trace!("Saving client connection {:?} ({:?})", k, v.user_id);
                    (k, v.save())
                })
                .collect(),
            flooded: self.flooded_connections.into_iter().map(ClientConnection::save).collect()
        }
    }

    /// Restore a collection from a previously stored state
    pub fn restore_from(state: ConnectionCollectionState,
                        listener_collection: &client_listener::ListenerCollection) -> Self
    {
        let mut ret = Self::new();

        for (conn_id, conn_data) in state.clients.into_iter()
        {
            let cli_conn = ClientConnection::restore(conn_data, listener_collection);
            if let Some(user_id) = &cli_conn.user_id
            {
                ret.add_user(*user_id, conn_id);
            }
            ret.client_connections.insert(conn_id, cli_conn);
        }
        ret.flooded_connections = state.flooded.into_iter()
                                               .map(|s| ClientConnection::restore(s, listener_collection))
                                               .collect();

        ret
    }
}

impl<'a> Iterator for UserConnectionIter<'a>
{
    type Item = &'a ClientConnection;

    fn next(&mut self) -> Option<Self::Item>
    {
        self.iter.as_mut().and_then(|it| it.next().and_then(|id| self.connections.get(id)))
    }
}

impl<'a> Iterator for UserConnectionIterMut<'a>
{
    type Item = &'a mut ClientConnection;

    fn next(&mut self) -> Option<&'a mut ClientConnection>
    {
        let mut_ref = self.iter.as_mut().and_then(|it| it.next().and_then(|id| self.connections.get_mut(id)));

        if let Some(mut_ref) = mut_ref
        {
            // Safety: this is unsafe because:
            //  * The lifetime of the returned mut reference is the lifetime of the iterator (i.e. `self`), but the
            //    returned type doesn't hold a mut borrow of `self`
            //  * As a result it's possible to call `next()` while still holding the mut ref returned from the
            //    previous call to `next()`
            //  * The compiler doesn't know that subsequent calls to `next()` won't return two mut refs to the same
            //    item.
            //
            // This is safe because:
            //   * We know that subsequent calls to `next()` on the same iterator won't return two mut refs to the
            //     same item
            //   * See the logic and comment in [`ConnectionCollection::add_user`]
            //
            // What we're achieving by converting to a pointer and back is to forcibly convert the reference
            // lifetime to the lifetime of the iterator. Given that we're guaranteeing uniqueness of items
            // returned by a single iterator, and the iterator itself holds a mut borrow of the collection,
            // this is safe to do.
            unsafe
            {
                let ptr: *mut ClientConnection = mut_ref;
                Some(&mut *ptr)
            }
        }
        else
        {
            None
        }
    }
}