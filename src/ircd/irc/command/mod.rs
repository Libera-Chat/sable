use super::*;
use crate::ircd::*;
use std::collections::HashMap;
use std::cell::RefCell;
use irc::numeric;
use irc::message;

use ircd_macros::command_handler;

mod processor;
pub use processor::*;

pub type CommandResult = Result<(), CommandError>;

pub trait CommandHandler
{
    fn min_parameters(&self) -> usize;

    fn validate(&self, cmd: &ClientCommand) -> CommandResult
    {
        if cmd.args.len() < self.min_parameters()
        {
            return numeric_error!(NotEnoughParameters, &cmd.command);
        }
        Ok(())
    }

    fn handle(&mut self, cmd: &ClientCommand) -> CommandResult
    {
        match &cmd.source {
            CommandSource::PreClient(pc) => {
                self.handle_preclient(pc, cmd)
            },
            CommandSource::User(u) => {
                self.handle_user(&u, cmd)
            }
        }
    }

    fn handle_preclient<'a>(&mut self, _source: &'a RefCell<PreClient>, _cmd: &ClientCommand) -> CommandResult
    {
        numeric_error!(NotRegistered)
    }

    fn handle_user<'a>(&mut self, _source: &'a wrapper::User, _cmd: &ClientCommand) -> CommandResult
    {
        numeric_error!(AlreadyRegistered)
    }
}

pub trait CommandHandlerFactory
{
    fn create<'a>(&self, server: &'a Server, proc: &'a CommandProcessor<'a>) -> Box<dyn CommandHandler + 'a>;
}

pub struct CommandRegistration
{
    command: String,
    handler: Box<dyn CommandHandlerFactory>,
}

pub struct CommandDispatcher
{
    handlers: HashMap<String, &'static Box<dyn CommandHandlerFactory>>
}

inventory::collect!(CommandRegistration);

impl CommandDispatcher {
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

    pub fn resolve_command(&self, cmd: &str) -> Option<&Box<dyn CommandHandlerFactory>>
    {
        self.handlers.get(&cmd.to_ascii_uppercase()).map(|x| *x)
    }
}

mod nick;
mod user;
mod join;
mod part;
mod privmsg;
mod quit;
mod mode;
mod ping;
mod names;
mod who;