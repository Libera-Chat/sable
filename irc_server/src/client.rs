use irc_network::*;
use super::*;
use client_listener::*;

use std::cell::RefCell;
use std::net::IpAddr;

pub struct ClientConnection
{
    pub connection: Connection,
    pub user_id: Option<UserId>,
    pub pre_client: Option<RefCell<PreClient>>
}

#[derive(serde::Serialize,serde::Deserialize)]
pub struct PreClient
{
    pub user: Option<Username>,
    pub nick: Option<Nickname>,
    pub realname: Option<String>,
    pub hostname: Option<Hostname>,
    pub cap_in_progress: bool,
}

impl ClientConnection
{
    pub fn new(conn: Connection) -> Self
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

    pub fn send(&self, msg: &dyn messages::MessageType)
    {
        self.connection.send(msg.to_string())
    }

    pub fn error(&self, msg: &str)
    {
        // Can't do much if this does fail, because we're already tearing down the connection
        let _res = self.connection.send(format!("ERROR :{}", msg));
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
            cap_in_progress: false
        }
    }

    pub fn can_register(&self) -> bool
    {
        self.user.is_some() && self.nick.is_some() && self.hostname.is_some() && !self.cap_in_progress
    }
}