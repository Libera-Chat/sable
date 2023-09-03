//! Command handlers.

use crate::capability::ClientCapabilitySet;

use super::*;
use client::*;
use messages::*;
use sable_network::prelude::*;

use std::{collections::HashMap, str::FromStr, sync::Arc};

mod client_command;
pub use client_command::*;

mod action;
pub use action::*;

mod error;
pub use error::*;

mod dispatcher;
pub use dispatcher::*;

mod plumbing;
pub use plumbing::{ArgListIter, Command};

/// A convenience definition for the result type returned from command handlers
pub type CommandResult = Result<(), CommandError>;

pub type AsyncHandler<'cmd> =
    std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync + 'cmd>>;

mod handlers {
    // These are here so the handler modules can import everything easily
    use super::*;
    use plumbing::*;
    use sable_macros::command_handler;
    use std::ops::Deref;

    mod admin;
    mod away;
    mod cap;
    mod chathistory;
    mod invite;
    mod join;
    mod kill;
    mod kline;
    mod mode;
    mod motd;
    mod names;
    mod nick;
    mod notice;
    mod oper;
    mod part;
    mod ping;
    mod pong;
    mod privmsg;
    mod quit;
    pub mod register;
    mod topic;
    mod user;
    mod who;
    mod whois;

    // Interim solutions that need refinement
    mod session;

    // Services compatibility command layer
    mod services;

    // Dev/test tools
    #[cfg(debug)]
    mod async_wait;
    #[cfg(debug)]
    mod sping;
}
