use crate::ircd::event::*;
use crate::ircd::*;

use std::collections::HashMap;

pub struct Network {
    users: HashMap<Id, state::User>,
    channels: HashMap<Id, state::Channel>,
    memberships: HashMap<Id, state::Membership>,
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
        match &event.details {
            EventDetails::NewUser(details) => self.new_user(event, details),
            EventDetails::NewChannel(details) => self.new_channel(event, details),
            EventDetails::ChannelJoin(details) => self.user_joined_channel(event, details),
        }
    }

}

mod accessors;

mod user_state;
mod channel_state;