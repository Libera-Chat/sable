use crate::ircd::*;
use crate::ircd::irc::CommandResult;
use irc::numeric;

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

    pub fn can_send(&self, user: &User) -> CommandResult
    {
        let mem_id = MembershipId::new(user.id(), self.data.id);
        let membership = self.network.membership(mem_id);
        if membership.is_err() && self.mode()?.has_mode(ChannelModeFlags::NO_EXTERNAL)
        {
            return Err(numeric::CannotSendToChannel::new(self).into());
        }
        Ok(())
    }
}

impl<'a> super::ObjectWrapper<'a> for Channel<'a> {
    type Underlying = state::Channel;

    fn wrap(net: &'a Network, data: &'a state::Channel) -> Self {
        Channel{ network: net, data: data }
    }
}
