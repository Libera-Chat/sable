use super::*;
use client_listener::ConnectionId;

pub(super) struct ConnectionCollection
{
    client_connections: HashMap<ConnectionId, ClientConnection>,
    user_to_connid: HashMap<UserId, ConnectionId>,
}

impl ConnectionCollection
{
    pub fn new() -> Self {
        Self {
            client_connections: HashMap::new(),
            user_to_connid: HashMap::new(),
        }
    }

    pub fn add(&mut self, id: ConnectionId, conn: ClientConnection)
    {
        self.client_connections.insert(id, conn);
    }

    pub fn add_user(&mut self, user: UserId, to: ConnectionId)
    {
        self.user_to_connid.insert(user, to);
    }

    pub fn remove(&mut self, id: ConnectionId)
    {
        if let Some(conn) = self.client_connections.get(&id)
        {
            if let Some(userid) = conn.user_id {
                self.user_to_connid.remove(&userid);
            }
        }
        self.client_connections.remove(&id);
    }

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

    pub fn get(&self, id: ConnectionId) -> Result<&ClientConnection, LookupError>
    {
        self.client_connections.get(&id).ok_or(LookupError::NoSuchConnectionId)
    }

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
    pub fn get_user_mut(&mut self, id: UserId) -> Result<&mut ClientConnection, LookupError>
    {
        match self.user_to_connid.get(&id) {
            None => Err(LookupError::NoSuchConnectionId),
            Some(connid) => self.client_connections.get_mut(connid).ok_or(LookupError::NoSuchConnectionId)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=&ClientConnection>
    {
        self.client_connections.values()
    }

    pub fn len(&self) -> usize
    {
        self.client_connections.len()
    }
}