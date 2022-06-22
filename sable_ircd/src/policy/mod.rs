// Most of the policy types have zero-parameter new(), but aren't a meaningful candidate for Default
#![allow(clippy::new_without_default)]

use sable_network::prelude::*;
use wrapper::*;

use crate::*;
use messages::Numeric;
use client::ClientConnection;

use ambassador::{
    delegatable_trait,
    Delegate
};

mod ban_resolver;
pub use ban_resolver::*;

#[macro_use]
mod channel_policy;
pub use channel_policy::*;

#[macro_use]
mod user_policy;
pub use user_policy::*;

#[macro_use]
mod oper_policy;
pub use oper_policy::*;

#[macro_use]
mod access_policy;
pub use access_policy::*;

mod standard_channel_policy;
pub use standard_channel_policy::*;

mod standard_user_policy;
pub use standard_user_policy::*;

mod standard_oper_policy;
pub use standard_oper_policy::*;

mod standard_access_policy;
pub use standard_access_policy::*;

mod error;
pub use error::*;

/// Convenience definition of the `Result` type for permission checks.
pub type PermissionResult = Result<(), PermissionError>;

/// A `PolicyService` provides all the various policy traits in one place
pub trait PolicyService:
            ChannelPolicyService +
            UserPolicyService +
            OperAuthenticationService +
            OperPolicyService +
            AccessPolicyService
{
}

/// The standard implementation of a [`PolicyService`]
#[derive(Delegate)]
#[delegate(ChannelPolicyService, target="channel_policy")]
#[delegate(UserPolicyService, target="user_policy")]
#[delegate(OperPolicyService, target="oper_policy")]
#[delegate(OperAuthenticationService, target="oper_policy")]
#[delegate(AccessPolicyService, target="access_policy")]
pub struct StandardPolicyService
{
    channel_policy: StandardChannelPolicy,
    user_policy: StandardUserPolicy,
    oper_policy: StandardOperPolicy,
    access_policy: StandardAccessPolicy,
}

impl StandardPolicyService
{
    pub fn new() -> Self {
        Self {
            channel_policy: StandardChannelPolicy::new(),
            user_policy: StandardUserPolicy::new(),
            oper_policy: StandardOperPolicy::new(),
            access_policy: StandardAccessPolicy::new(),
        }
    }
}

impl PolicyService for StandardPolicyService
{
}
