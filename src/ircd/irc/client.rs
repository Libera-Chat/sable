use crate::ircd::*;
use super::*;

use std::cell::RefCell;
use async_std::net::IpAddr;

pub struct ClientConnection
{
    pub connection: connection::Connection,
    pub user_id: Option<UserId>,
    pub pre_client: Option<RefCell<PreClient>>
}

pub struct PreClient
{
    pub user: Option<Username>,
    pub nick: Option<Nickname>,
    pub realname: Option<String>,
    pub hostname: Option<Hostname>,
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

    pub fn remote_addr(&self) -> IpAddr
    {
        self.connection.remote_addr
    }

    pub fn send(&self, msg: &dyn messages::Message)
    {
        if let Err(e) = self.connection.send(&msg.to_string())
        {
            self.error(&e.to_string())
        }
    }

    pub fn error(&self, msg: &str)
    {
        // Can't do much if this does fail, because we're already tearing down the connection
        let _res = self.connection.send(&format!("ERROR :{}", msg));
        let _res = self.connection.close();
    }
}

impl PreClient {
    pub fn new() -> Self
    {
        Self {
            user: None,
            nick: None,
            realname: None,
            hostname: None,
        }
    }

    pub fn can_register(&self) -> bool
    {
        self.user.is_some() && self.nick.is_some() && self.hostname.is_some()
    }
}