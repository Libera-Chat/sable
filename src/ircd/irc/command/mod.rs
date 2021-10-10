use super::*;

pub trait CommandHandler
{
    fn handle(&self, server: &Server, cmd: &ClientCommand, actions: &mut Vec<CommandAction>) -> Result<(), CommandError>;
    fn min_parameters(&self) -> usize;

    fn validate(&self, _server: &Server, cmd: &ClientCommand) -> Result<(), CommandError>
    {
        if cmd.args.len() < self.min_parameters()
        {
            return Err(CommandError::NotEnoughParameters);
        }
        Ok(())
    }
}

pub fn resolve_command(cmd: &str) -> Option<Box<dyn CommandHandler>>
{
    match cmd.to_ascii_uppercase().as_str() {
        "NICK" => Some(Box::new(nick::NickHandler())),
        "USER" => Some(Box::new(user::UserHandler())),
        _ => None
    }
}

mod nick;
mod user;
mod join;
mod message;