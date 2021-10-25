use crate::ircd::*;
use wrapper::*;
use irc::messages::Numeric;

pub type PermissionResult = Result<(), PermissionError>;

pub trait PolicyService: ChannelPolicyService
{
}

pub struct StandardPolicyService
{
}

impl StandardPolicyService
{
    pub fn new() -> Self {
        Self { }
    }
}

impl PolicyService for StandardPolicyService
{
}

mod channel_access;
pub use channel_access::*;

mod error;
pub use error::*;