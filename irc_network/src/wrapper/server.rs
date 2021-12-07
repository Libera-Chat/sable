use crate::*;
use super::*;

/// A wrapper around a [`state::Server`]
pub struct Server<'a> {
    network: &'a Network,
    data: &'a state::Server,
}

impl Server<'_> {
    /// Return this object's ID
    pub fn id(&self) -> ServerId {
        self.data.id
    }

    /// This server's current epoch
    pub fn epoch(&self) -> EpochId {
        self.data.epoch
    }

    /// The server's name
    pub fn name(&self) -> &ServerName {
        &self.data.name
    }

    /// The timestamp of the last ping received from this server
    pub fn last_ping(&self) -> i64 {
        self.data.last_ping
    }

    /// The event ID by which this server was introduced to the network
    pub fn introduced_by(&self) -> EventId {
        self.data.introduced_by
    }

    /// Iterate over the users connecting via this server
    pub fn users(&self) -> impl Iterator<Item=User>
    {
        let id = self.data.id;
        self.network.raw_users().filter(move |u| u.server == id).wrap(self.network)
    }
}

impl<'a> super::ObjectWrapper<'a> for Server<'a> {
    type Underlying = state::Server;

    fn wrap(net: &'a Network, data: &'a state::Server) -> Self {
        Self{network: net, data: data}
    }
}