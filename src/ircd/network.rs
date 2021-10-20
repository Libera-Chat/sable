use crate::ircd::event::*;
use crate::ircd::*;
use crate::utils::{OrLog,FlattenResult};
use ircd_macros::dispatch_event;
use thiserror::Error;

use std::collections::HashMap;

#[derive(Error,Debug)]
pub enum ValidationError
{
    #[error("Nickname {0} already in use")]
    NickInUse(String),
    #[error("Object not found: {0}")]
    ObjectNotFound(#[from] LookupError),
    #[error("Wrong object ID type: {0}")]
    WrongTypeId(#[from] WrongIdTypeError)
}
pub type ValidationResult = Result<(), ValidationError>;

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
        dispatch_event!(event => {
            NewUser => self.new_user,
            UserQuit => self.user_quit,
            NewChannel => self.new_channel,
            ChannelJoin => self.user_joined_channel,
            ChannelPart => self.user_left_channel,
            NewMessage => self.new_message,
        }).or_log("Mismatched object ID type?");
    }

    pub fn validate(&self, event: &Event) -> ValidationResult
    {
        dispatch_event!(event => {
            NewUser => self.validate_new_user,
            _ => (|_| { Ok(()) })
        }).flatten()
    }
}

mod accessors;

mod user_state;
mod channel_state;
mod message_state;