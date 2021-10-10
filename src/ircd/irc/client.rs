use crate::ircd::*;
use super::*;

use std::cell::RefCell;

pub struct ClientConnection
{
    pub connection: connection::Connection,
    pub user_id: Option<Id>,
    pub pre_client: Option<PreClient>
}

pub struct PreClient
{
    pub user: RefCell<Option<String>>,
    pub nick: RefCell<Option<String>>
}

impl ClientConnection
{
    pub fn new(conn: connection::Connection) -> Self
    {
        Self {
            connection: conn,
            user_id: None,
            pre_client: Some(PreClient::new())
        }
    }

    pub fn id(&self) -> Id
    {
        self.connection.id
    }

    
}

impl PreClient {
    pub fn new() -> Self
    {
        Self {
            user: RefCell::new(None),
            nick: RefCell::new(None)
        }
    }

    pub fn can_register(&self) -> bool
    {
        self.user.borrow().is_some() && self.nick.borrow().is_some()
    }
}