use crate::ircd::*;
use super::*;

use std::cell::RefCell;

pub struct ClientConnection
{
    pub connection: connection::Connection,
    pub user_id: Option<UserId>,
    pub pre_client: Option<RefCell<PreClient>>
}

pub struct PreClient
{
    pub user: Option<String>,
    pub nick: Option<String>,
    pub realname: Option<String>,
}

impl ClientConnection
{
    pub fn new(conn: connection::Connection) -> Self
    {
        Self {
            connection: conn,
            user_id: None,
            pre_client: Some(RefCell::new(PreClient::new()))
        }
    }

    pub fn id(&self) -> ConnectionId
    {
        self.connection.id
    }
}

impl PreClient {
    pub fn new() -> Self
    {
        Self {
            user: None,
            nick: None,
            realname: None,
        }
    }

    pub fn can_register(&self) -> bool
    {
        self.user.is_some() && self.nick.is_some()
    }
}