use irc_network::*;
use wrapper::*;
use crate::Numeric;

use ambassador::{
    delegatable_trait,
    Delegate
};

mod ban_resolver;
pub use ban_resolver::*;

#[macro_use]
mod channel_policy;
pub use channel_policy::*;

mod standard_channel_policy;
pub use standard_channel_policy::*;

mod error;
pub use error::*;

pub type PermissionResult = Result<(), PermissionError>;

pub trait PolicyService: ChannelPolicyService
{
}

#[derive(Delegate)]
#[delegate(ChannelPolicyService)]
pub struct StandardPolicyService
{
    channel_policy: StandardChannelPolicy,
}

impl StandardPolicyService
{
    pub fn new() -> Self {
        Self {
            channel_policy: StandardChannelPolicy::new()    
        }
    }
}

impl PolicyService for StandardPolicyService
{
}
