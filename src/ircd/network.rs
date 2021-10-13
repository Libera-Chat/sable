use crate::ircd::event::*;
use crate::ircd::*;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Network {
    users: HashMap<UserId, state::User>,
    channels: HashMap<ChannelId, state::Channel>,
    memberships: HashMap<MembershipId, state::Membership>,

    messages: HashMap<MessageId, state::Message>,
}

impl Network {
    pub fn new() -> Network {
        Network{
            users: HashMap::new(),
            channels: HashMap::new(),
            memberships: HashMap::new(),

            messages: HashMap::new(),
        }
    }

    pub fn apply(&mut self, event: &Event) {
        match (event.target, &event.details) {
            (ObjectId::User(target), EventDetails::NewUser(details)) => self.new_user(target, event, details),
            (ObjectId::Channel(target), EventDetails::NewChannel(details)) => self.new_channel(target, event, details),
            (ObjectId::Membership(target), EventDetails::ChannelJoin(details)) => self.user_joined_channel(target, event, details),
            (ObjectId::Message(target), EventDetails::NewMessage(details)) => self.new_message(target, event, details),
            _ => panic!("Network received event with wrong target type: {:?}, {:?}", event.target, event.details)
        }
    }
}

mod errors;
pub use errors::LookupError;
pub use errors::LookupResult;

mod accessors;

mod user_state;
mod channel_state;
mod message_state;