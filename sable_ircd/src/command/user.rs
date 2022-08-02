use super::*;

command_handler!("USER" => UserHandler {
    fn min_parameters(&self) -> usize { 4 }

    fn handle_preclient(&mut self, source: &PreClient, cmd: &ClientCommand) -> CommandResult
    {
        // Ignore these results; they'll only fail if USER was already successfully processed
        // from this pre-client. If that happens we silently ignore the new values.
        source.user.set(Username::new_coerce(&cmd.args[0])).ok();
        source.realname.set(cmd.args[3].clone()).ok();

        if source.can_register()
        {
            self.action(CommandAction::RegisterClient(cmd.connection.id()))?;
        }
        Ok(())
    }
});
