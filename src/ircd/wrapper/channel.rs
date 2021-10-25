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

    pub fn mode(&self) -> LookupResult<ChannelMode> {
        self.network.channel_mode(self.data.mode)
    }

    pub fn members(&self) -> impl Iterator<Item=Membership> {
        let my_id = self.data.id;
        self.network.raw_memberships().filter(move |x| x.channel == my_id).wrap(self.network)
    }

    pub fn has_member(&self, u: UserId) -> Option<Membership>
    {
        self.members().filter(|m| m.user_id() == u).next()
    }
}

impl<'a> super::ObjectWrapper<'a> for Channel<'a> {
    type Underlying = state::Channel;

    fn wrap(net: &'a Network, data: &'a state::Channel) -> Self {
        Channel{ network: net, data: data }
    }
}
