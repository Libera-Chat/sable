use super::Network;
use crate::ircd::*;
use crate::ircd::event::*;

impl Network {
    pub fn new_channel(&mut self, event: &Event, details: &details::NewChannel) {
        let channel = state::Channel::new(event.target, &details.name);
        self.channels.insert(channel.id, channel);
    }

    pub fn user_joined_channel(&mut self, event: &Event, details: &details::ChannelJoin) {
        let membership = state::Membership::new(event.target, details.user, details.channel);
        self.memberships.insert(membership.id, membership);
    }
}