use super::Network;
use crate::ircd::*;
use crate::ircd::event::*;

impl Network {
    pub fn new_channel(&mut self, target: ChannelId, _event: &Event, details: &details::NewChannel) {
        let channel = state::Channel::new(target, &details.name);
        self.channels.insert(channel.id, channel);
    }

    pub fn user_joined_channel(&mut self, target: MembershipId, _event: &Event, details: &details::ChannelJoin) {
        let membership = state::Membership::new(target, details.user, details.channel);
        self.memberships.insert(membership.id, membership);
    }
}