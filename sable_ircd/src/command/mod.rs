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
pub use plumbing::{ArgListIter, Command, LoggedInUserSource, PreClientSource, UserSource};

/// A convenience definition for the result type returned from command handlers
pub type CommandResult = Result<(), CommandError>;

pub type AsyncHandler<'cmd> =
    std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'cmd>>;

mod handlers {
    // These are here so the handler modules can import everything easily
    use super::*;
    use plumbing::*;
    use sable_macros::command_handler;
    use std::ops::Deref;

    mod admin;
    mod away;
    mod ban;
    mod cap;
    mod chathistory;
    mod info;
    mod invite;
    mod join;
    mod kick;
    mod kill;
    mod kline;
    mod links;
    mod mode;
    mod monitor;
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
    mod rename;
    mod tagmsg;
    mod topic;
    mod user;
    mod userhost;
    mod version;
    mod who;
    mod whois;
    mod whowas;

    // Interim solutions that need refinement
    mod session;

    // Services compatibility command layer
    mod services;
}
