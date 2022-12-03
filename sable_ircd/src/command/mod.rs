//! Command handlers.

use crate::capability::ClientCapabilitySet;

use super::*;
use sable_network::prelude::*;
use messages::*;
use client::*;
use crate::utils::ClientCommandExt;

use std::{
    collections::HashMap,
    sync::Arc,
};

use command_processor::*;

/// A convenience definition for the result type returned from command handlers
pub type CommandResult = Result<(), CommandError>;

/// The trait to be implemented by command handler objects.
///
/// This will usually be implemented via the `command_handler!` macro; see the
/// various existing command handlers for examples.
pub(crate) trait CommandHandler
{
    /// Define the minimum number of parameters required for this command.
    fn min_parameters(&self) -> usize;

    /// Define any client capabilities which must be negotiated before this command
    /// can be used
    fn required_capabilities(&self) -> ClientCapabilitySet
    {
        ClientCapabilitySet::new()
    }

    /// Perform any low-cost validation that may be appropriate before invoking the
    /// relevant handler function. If validation fails, an appropriate `Err` value should
    /// be returned.
    ///
    /// The default implementation:
    ///  * checks whether the handler requires specific client capabilities, and returns an error
    ///    if these are not met
    ///  * checks the number of provided parameters against the result of `self.min_parameters()`,
    ///    returning an appropriate error numeric if insufficient parameters were provided
    fn validate(&self, cmd: &ClientCommand) -> CommandResult
    {
        if ! cmd.connection.capabilities.has_all(self.required_capabilities())
        {
            return numeric_error!(UnknownCommand, &cmd.command);
        }
        if cmd.args.len() < self.min_parameters()
        {
            return numeric_error!(NotEnoughParameters, &cmd.command);
        }
        Ok(())
    }

    /// Handle the command, from any source.
    ///
    /// The default implementation invokes [`Self::handle_preclient`] or [`Self::handle_user`]
    /// depending on the status of the source connection.
    fn handle(&mut self, cmd: &ClientCommand) -> CommandResult
    {
        match cmd.source() {
            CommandSource::PreClient(pc) => {
                self.handle_preclient(&*pc, cmd)
            },
            CommandSource::User(u) => {
                self.handle_user(&u, cmd)
            }
        }
    }

    /// Handle the command asynchronously, from any source.
    ///
    /// The default implementation delegates to either `handle_preclient_async` or `handle_user_async`
    /// depending on the status of the source connection.
    ///
    /// If this method returns a future, that future will be polled by the server run loop and executed
    /// as the command handler. If it returns `None` (as the default implementation does), then the
    /// command will be handled synchronously (by calling
    fn handle_async<'a>(&mut self, cmd: Arc<ClientCommand>) -> Option<server::AsyncHandler<'a>>
    {
        match cmd.source() {
            CommandSource::PreClient(pc) => {
                self.handle_preclient_async(pc, cmd)
            },
            CommandSource::User(u) => {
                self.handle_user_async(u.id(), cmd)
            }
        }
    }

    /// Handle the command when it originates from a client connection that has not completed
    /// registration.
    ///
    /// The default produces an error numeric instructing the client to register.
    fn handle_preclient<'a>(&mut self, _source: &'a PreClient, _cmd: &ClientCommand) -> CommandResult
    {
        numeric_error!(NotRegistered)
    }

    /// Handle the command asynchronously, when it originates from a client connection that has not
    /// completed registration. Implementations should return a boxed Future which will be driven by
    /// the server run loop.
    ///
    /// The default returns `None`, indicating that the command should be handled synchronously
    fn handle_preclient_async<'a>(&mut self, _source: Arc<PreClient>, _cmd: Arc<ClientCommand>) -> Option<server::AsyncHandler<'a>>
    {
        None
    }

    /// Handle the command when it originates from a registered user connection.
    ///
    /// If not implemented, the default is to return a numeric error indicating that the user
    /// has already registered and cannot do so again.
    fn handle_user<'a>(&mut self, _source: &'a wrapper::User, _cmd: &ClientCommand) -> CommandResult
    {
        numeric_error!(AlreadyRegistered)
    }

    /// Handle the command asynchronously, when it originates from a registered user connection.
    /// Implementations should return a boxed Future which will be driven by the server run loop.
    ///
    /// The default returns `None`, indicating that the command should be handled synchronously
    fn handle_user_async<'a>(&mut self, _source: UserId, _cmd: Arc<ClientCommand>) -> Option<server::AsyncHandler<'a>>
    {
        None
    }

    fn handle_oper<'a>(&mut self, source: &'a wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        self.handle_user(source, cmd)
    }
}

type CommandHandlerFactory = fn(Arc<ClientServer>) -> Box<dyn CommandHandler>;

/// A command handler registration. Constructed by the `command_handler` macro.
pub(crate) struct CommandRegistration
{
    command: &'static str,
    handler: CommandHandlerFactory,
}

/// A command dispatcher. Collects registered command handlers and allows lookup by
/// command name.
pub(crate) struct CommandDispatcher
{
    handlers: HashMap<String, CommandHandlerFactory>
}

inventory::collect!(CommandRegistration);

impl CommandDispatcher {
    /// Construct a `CommandDispatcher`.
    ///
    /// Handlers are populated via compile-time registration.
    pub fn new() -> Self
    {
        let mut map = HashMap::new();

        for reg in inventory::iter::<CommandRegistration> {
            map.insert(reg.command.to_ascii_uppercase(), reg.handler);
        }

        Self {
            handlers: map
        }
    }

    /// Look up the handler factory corresponding to a given command.
    // &Box actually makes sense for a return value given the type in the hashmap
    #[allow(clippy::borrowed_box)]
    pub fn resolve_command(&self, cmd: &str) -> Option<CommandHandlerFactory>
    {
        self.handlers.get(&cmd.to_ascii_uppercase()).copied()
    }
}

macro_rules! command_handler {
    ($cmd:literal => $typename:ident $body:tt) =>
    {
        struct $typename
        {
            server: Arc<crate::server::ClientServer>,
        }

        impl $typename
        {
            pub fn new(server: Arc<crate::server::ClientServer>) -> Self
            {
                Self{ server }
            }

            // Not all handlers will actually use this, which is OK
            #[allow(dead_code)]
            pub fn action(&mut self, act: crate::command_processor::CommandAction) -> sable_network::network::ValidationResult
            {
                if let CommandAction::StateChange(i, d) = &act {
                    self.server.network().validate(*i, d)?;
                }
                self.server.add_action(act);
                Ok(())
            }
        }

        impl CommandHandler for $typename
        $body

        mod registration
        {
            // macro_rules macros can't modify identifiers they're given, so we're stuck with this
            #[allow(non_snake_case)]
            mod $typename
            {
                fn factory_function(server: std::sync::Arc<crate::server::ClientServer>) -> Box<dyn crate::command::CommandHandler>
                {
                    Box::new(super::super::$typename::new(server))
                }

                inventory::submit!(crate::command::CommandRegistration {
                    command: $cmd,
                    handler: factory_function
                });
            }
        }
    }
}

mod cap;
mod nick;
mod user;
mod join;
mod part;
mod notice;
mod privmsg;
mod quit;
mod mode;
mod ping;
mod names;
mod who;
mod whois;
mod topic;
mod invite;
mod kill;
mod kline;
mod oper;
mod chathistory;
mod register;

// Interim solutions that need refinement
mod session;

// Services compatibility command layer
mod ns;
mod cs;

// Dev/test tools
#[cfg(debug)]
mod async_wait;
#[cfg(debug)]
mod sping;
