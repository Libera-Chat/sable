use crate::ircd::event::*;
use crate::ircd::*;
use crate::utils::OrLog;
use ircd_macros::dispatch_event;

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
/*
    pub fn apply(&mut self, event: &Event) {
        match (event.target, &event.details) {
            (ObjectId::User(target), EventDetails::NewUser(details)) => self.new_user(target, event, details),
            (ObjectId::Channel(target), EventDetails::NewChannel(details)) => self.new_channel(target, event, details),
            (ObjectId::Membership(target), EventDetails::ChannelJoin(details)) => self.user_joined_channel(target, event, details),
            (ObjectId::Message(target), EventDetails::NewMessage(details)) => self.new_message(target, event, details),
            _ => panic!("Network received event with wrong target type: {:?}, {:?}", event.target, event.details)
        }
    }
*/

    pub fn apply(&mut self, event: &Event) {
        dispatch_event!(event => {
            NewUser => self.new_user,
            UserQuit => self.user_quit,
            NewChannel => self.new_channel,
            ChannelJoin => self.user_joined_channel,
            NewMessage => self.new_message,
        }).or_log("Mismatched object ID type?");
    }
}

mod accessors;

mod user_state;
mod channel_state;
mod message_state;