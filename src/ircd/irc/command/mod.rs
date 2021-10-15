use super::*;
use crate::ircd::*;
use std::collections::HashMap;
use std::cell::RefCell;

use ircd_macros::command_handler;

type CommandResult = Result<(), CommandError>;

pub trait CommandHandler
{
    fn min_parameters(&self) -> usize;

    fn validate(&self, _server: &Server, cmd: &ClientCommand) -> CommandResult
    {
        if cmd.args.len() < self.min_parameters()
        {
            return Err(CommandError::NotEnoughParameters);
        }
        Ok(())
    }

    fn handle(&self, server: &Server, cmd: &ClientCommand, actions: &mut Vec<CommandAction>) -> CommandResult
    {
        match &cmd.source {
            CommandSource::PreClient(pc) => {
                self.handle_preclient(server, pc, cmd, actions)
            },
            CommandSource::User(u) => {
                self.handle_user(server, &u, cmd, actions)
            }
        }
    }

    fn handle_preclient<'a>(&self, _server: &Server, _source: &'a RefCell<PreClient>, _cmd: &ClientCommand, _actions: &mut Vec<CommandAction>) -> CommandResult
    {
        Err(CommandError::NotRegistered)
    }

    fn handle_user<'a>(&self, _server: &Server, _source: &'a wrapper::User, _cmd: &ClientCommand, _actions: &mut Vec<CommandAction>) -> CommandResult
    {
        Err(CommandError::AlreadyRegistered)
    }
}

pub struct CommandRegistration
{
    command: String,
    handler: Box<dyn CommandHandler>
}

pub struct CommandDispatcher
{
    handlers: HashMap<String, &'static Box<dyn CommandHandler>>
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

    pub fn resolve_command(&self, cmd: &str) -> Option<&Box<dyn CommandHandler>>
    {
        self.handlers.get(&cmd.to_ascii_uppercase()).map(|x| *x)
    }
}
mod nick;
mod user;
mod join;
mod message;
mod quit;
