use crate::event::*;
use crate::*;
use ircd_macros::dispatch_event;
use thiserror::Error;

use std::collections::HashMap;
use std::convert::TryInto;

use crate as irc_network;

#[derive(Error,Debug)]
pub enum ValidationError
{
    #[error("Nickname {0} already in use")]
    NickInUse(Nickname),
    #[error("Object not found: {0}")]
    ObjectNotFound(#[from] LookupError),
    #[error("Wrong object ID type: {0}")]
    WrongTypeId(#[from] WrongIdTypeError),
    #[error("{0}")]
    InvalidNickname(#[from]InvalidNicknameError),
    #[error("{0}")]
    InvalidUsername(#[from]InvalidUsernameError),
    #[error("{0}")]
    InvalidHostname(#[from]InvalidHostnameError),
    #[error("{0}")]
    InvalidChannelName(#[from]InvalidChannelNameError),
}
pub type ValidationResult = Result<(), ValidationError>;

#[derive(Debug)]
pub struct Network {
    users: HashMap<UserId, state::User>,
    user_modes: HashMap<UModeId, state::UserMode>,

    channels: HashMap<ChannelId, state::Channel>,
    channel_modes: HashMap<CModeId, state::ChannelMode>,

    memberships: HashMap<MembershipId, state::Membership>,

    messages: HashMap<MessageId, state::Message>,
}

impl Network {
    pub fn new() -> Network {
        Network{
            users: HashMap::new(),
            user_modes: HashMap::new(),

            channels: HashMap::new(),
            channel_modes: HashMap::new(),
            memberships: HashMap::new(),

            messages: HashMap::new(),
        }
    }

    pub fn apply(&mut self, event: &Event) -> Result<(),WrongIdTypeError> {
        dispatch_event!(event => {
            NewUser => self.new_user,
            UserNickChange => self.user_nick_change,
            UserQuit => self.user_quit,
            NewUserMode => self.new_user_mode,
            UserModeChange => self.user_mode_change,
            NewChannel => self.new_channel,
            NewChannelMode => self.new_channel_mode,
            ChannelModeChange => self.channel_mode_change,
            ChannelPermissionChange => self.channel_permission_change,
            ChannelJoin => self.user_joined_channel,
            ChannelPart => self.user_left_channel,
            NewMessage => self.new_message,
        })
    }

    pub fn validate(&self, id: ObjectId, detail: &EventDetails) -> ValidationResult
    {
        match detail {
            EventDetails::NewUser(newuser) => self.validate_new_user(id.try_into()?, newuser),
            EventDetails::UserNickChange(nickchange) => self.validate_nick_change(id.try_into()?, nickchange),
            _ => Ok(())
        }
    }
}

mod accessors;

mod user_state;
mod channel_state;
mod message_state;