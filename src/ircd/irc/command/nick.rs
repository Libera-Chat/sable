use super::*;

pub struct NickHandler();

impl CommandHandler for NickHandler
{
    fn min_parameters(&self) -> usize { 1 }

    fn handle(&self, _server: &Server, cmd: &ClientCommand, actions: &mut Vec<CommandAction>) -> Result<(), CommandError>
    {
        match &cmd.source {
            CommandSource::PreClient(c) => {
                c.nick.replace(Some(cmd.args[0].clone()));
                if c.can_register()
                {
                    actions.push(CommandAction::RegisterClient(cmd.connection.id()));
                }
            },
            CommandSource::User(_u) => {
                panic!("not implemented");
            }
        }
        Ok(())
    }
}