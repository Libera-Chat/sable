use std::net::IpAddr;

use super::*;
use crate::prelude::*;

/// A wrapper around a [`state::UserConnection`]
pub struct UserConnection<'a> {
    network: &'a Network,
    data: &'a state::UserConnection,
}

impl UserConnection<'_> {
    pub fn id(&self) -> UserConnectionId {
        self.data.id
    }

    pub fn user(&self) -> LookupResult<User> {
        self.network.user(self.data.user)
    }

    pub fn ip(&self) -> &IpAddr {
        &self.data.ip
    }

    pub fn hostname(&self) -> &Hostname {
        &self.data.hostname
    }

    pub fn server(&self) -> LookupResult<Server> {
        self.network.server(self.data.id.server())
    }
}

impl<'a> super::ObjectWrapper<'a> for UserConnection<'a> {
    type Underlying = state::UserConnection;

    fn wrap(network: &'a Network, data: &'a state::UserConnection) -> Self {
        Self { network, data }
    }

    fn raw(&self) -> &'a Self::Underlying {
        self.data
    }
}
