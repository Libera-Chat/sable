use super::*;

command_handler!("QUIT", QuitHandler);

impl CommandHandler for QuitHandler
{
    fn min_parameters(&self) -> usize { 0 }

    fn handle_user(&self, server: &Server, source: &wrapper::User, cmd: &ClientCommand, proc: &mut CommandProcessor) -> CommandResult
    {
        proc.action(CommandAction::DisconnectUser(source.id())).translate(cmd)?;
        proc.action(CommandAction::StateChange(server.create_event(
            source.id(),
            event::UserQuit { message: cmd.args.get(0).unwrap_or(&"Client Quit".to_string()).clone() }
        ))).translate(cmd)?;
        Ok(())
    }
}