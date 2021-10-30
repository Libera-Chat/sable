use super::*;

command_handler!("QUIT" => QuitHandler {
    fn min_parameters(&self) -> usize { 0 }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        self.action(CommandAction::DisconnectUser(source.id()))?;
        self.action(CommandAction::state_change(
            source.id(),
            event::UserQuit { message: cmd.args.get(0).unwrap_or(&"Client Quit".to_string()).clone() }
        ))?;
        Ok(())
    }
});