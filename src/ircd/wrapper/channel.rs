use crate::ircd::*;

use super::*;

pub struct Channel<'a> {
    network: &'a Network,
    data: &'a state::Channel,
}

impl Channel<'_> {
    pub fn id(&self) -> ChannelId {
        self.data.id
    }
    
    pub fn name(&self) -> &str {
        &self.data.name.value()
    }

    pub fn members(&self) -> impl Iterator<Item=Membership> {
        let my_id = self.data.id;
        self.network.raw_memberships().filter(move |x| x.channel == my_id).wrap(self.network)
    }
}

impl<'a> super::ObjectWrapper<'a> for Channel<'a> {
    type Underlying = state::Channel;

    fn wrap(net: &'a Network, data: &'a state::Channel) -> Self {
        Channel{ network: net, data: data }
    }
}