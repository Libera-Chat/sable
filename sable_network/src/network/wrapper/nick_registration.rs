use crate::prelude::*;

pub struct NickRegistration<'a> {
    _network: &'a Network,
    data: &'a state::NickRegistration,
}

impl<'a> super::ObjectWrapper<'a> for NickRegistration<'a> {
    type Underlying = state::NickRegistration;

    fn wrap(net: &'a Network, data: &'a Self::Underlying) -> Self
    {
        Self{ _network: net, data }
    }

    fn raw(&self) -> &'a Self::Underlying { self.data }
}

