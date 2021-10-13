use crate::ircd::event::*;
use crate::ircd::*;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Network {
    users: HashMap<UserId, state::User>,
    channels: HashMap<ChannelId, state::Channel>,
    memberships: HashMap<MembershipId, state::Membership>,
}

impl Network {
    pub fn new() -> Network {
        Network{
            users: HashMap::new(),
            channels: HashMap::new(),
            memberships: HashMap::new()
        }
    }

    pub fn apply(&mut self, event: &Event) {
        match (event.target, &event.details) {
            (ObjectId::User(target), EventDetails::NewUser(details)) => self.new_user(target, event, details),
            (ObjectId::Channel(target), EventDetails::NewChannel(details)) => self.new_channel(target, event, details),
            (ObjectId::Membership(target), EventDetails::ChannelJoin(details)) => self.user_joined_channel(target, event, details),
            _ => panic!("Network received event with wrong target type: {:?}, {:?}", event.target, event.details)
        }
    }
}

mod accessors;

mod user_state;
mod channel_state;