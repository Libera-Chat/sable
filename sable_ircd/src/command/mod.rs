use super::*;
use sable_network::prelude::*;
use messages::*;
use client::*;

use std::collections::HashMap;
use std::cell::RefCell;

use sable_macros::command_handler;

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

    /// Perform any low-cost validation that may be appropriate before invoking the
    /// relevant handler function. If validation fails, an appropriate `Err` value should
    /// be returned.
    ///
    /// The default implementation simply checks the number of provided parameters
    /// against the result of `self.min_parameters()`, returning an appropriate error
    /// numeric if insufficient parameters were provided.
    fn validate(&self, cmd: &ClientCommand) -> CommandResult
    {
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
        match &cmd.source {
            CommandSource::PreClient(pc) => {
                self.handle_preclient(pc, cmd)
            },
            CommandSource::User(u) => {
                self.handle_user(u, cmd)
            }
        }
    }

    /// Handle the command when it originates from a client connection that has not completed
    /// registration.
    ///
    /// The default produces an error numeric instructing the client to register.
    fn handle_preclient<'a>(&mut self, _source: &'a RefCell<PreClient>, _cmd: &ClientCommand) -> CommandResult
    {
        numeric_error!(NotRegistered)
    }

    /// Handle the command when it originates from a registered user connection.
    ///
    /// If not implemented, the default is to return a numeric error indicating that the user
    /// has already registered and cannot do so again.
    fn handle_user<'a>(&mut self, _source: &'a wrapper::User, _cmd: &ClientCommand) -> CommandResult
    {
        numeric_error!(AlreadyRegistered)
    }

    fn handle_oper<'a>(&mut self, source: &'a wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        self.handle_user(source, cmd)
    }
}

/// Factory trait to construct command handlers. Implemented internally by the `command_handler`
/// macro; there is generally no need to implement this in application code.
pub(crate) trait CommandHandlerFactory
{
    /// Create an object implementing [`CommandHandler`].
    fn create<'a>(&self, server: &'a ClientServer, proc: &'a CommandProcessor<'a>) -> Box<dyn CommandHandler + 'a>;
}

/// A command handler registration. Constructed by the `command_handler` macro.
pub(crate) struct CommandRegistration
{
    command: String,
    handler: Box<dyn CommandHandlerFactory>,
}

/// A command dispatcher. Collects registered command handlers and allows lookup by
/// command name.
pub(crate) struct CommandDispatcher
{
    #[allow(clippy::borrowed_box)]
    handlers: HashMap<String, &'static Box<dyn CommandHandlerFactory>>
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
            map.insert(reg.command.to_ascii_uppercase(), &reg.handler);
        }

        Self {
            handlers: map
        }
    }

    /// Look up the handler factory corresponding to a given command.
    // &Box actually makes sense for a return value given the type in the hashmap
    #[allow(clippy::borrowed_box)]
    pub fn resolve_command(&self, cmd: &str) -> Option<&Box<dyn CommandHandlerFactory>>
    {
        self.handlers.get(&cmd.to_ascii_uppercase()).copied()
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