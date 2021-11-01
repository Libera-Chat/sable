use crate::*;
use super::*;

pub struct Server<'a> {
    network: &'a Network,
    data: &'a state::Server,
}

impl Server<'_> {
    pub fn id(&self) -> ServerId {
        self.data.id
    }

    pub fn name(&self) -> &ServerName {
        &self.data.name
    }

    pub fn last_ping(&self) -> i64 {
        self.data.last_ping
    }

    pub fn introduced_by(&self) -> EventId {
        self.data.introduced_by
    }

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