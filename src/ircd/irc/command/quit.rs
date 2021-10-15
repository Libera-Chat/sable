use super::*;

command_handler!("QUIT", QuitHandler);

impl CommandHandler for QuitHandler
{
    fn min_parameters(&self) -> usize { 0 }

    fn handle_user(&self, server: &Server, source: &wrapper::User, cmd: &ClientCommand, actions: &mut Vec<CommandAction>) -> CommandResult
    {
        actions.push(CommandAction::DisconnectUser(source.id()));
        actions.push(CommandAction::StateChange(server.create_event(
            source.id(),
            event::UserQuit { message: cmd.args.get(0).unwrap_or(&"Client Quit".to_string()).clone() }
        )));
        Ok(())
    }
}