use super::*;
use crate::prelude::*;

/// A wrapper around a [`state::ServicesData`]
pub struct ServicesData<'a> {
    network: &'a Network,
    data: &'a state::ServicesData,
}

impl ServicesData<'_> {
    pub fn server(&self) -> LookupResult<Server> {
        self.network.server(self.data.server_id)
    }

    pub fn server_id(&self) -> ServerId {
        self.data.server_id
    }

    pub fn server_name(&self) -> LookupResult<ServerName> {
        self.server().map(|s| s.name().clone())
    }

    pub fn sasl_mechanisms(&self) -> &Vec<String> {
        &self.data.sasl_mechanisms
    }
}

impl<'a> super::ObjectWrapper<'a> for ServicesData<'a> {
    type Underlying = state::ServicesData;

    fn wrap(network: &'a Network, data: &'a state::ServicesData) -> Self {
        Self { network, data }
    }

    fn raw(&self) -> &'a Self::Underlying {
        self.data
    }
}
