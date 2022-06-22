use super::*;

command_handler!("USER" => UserHandler {
    fn min_parameters(&self) -> usize { 4 }

    fn handle_preclient(&mut self, source: &RefCell<PreClient>, cmd: &ClientCommand) -> CommandResult
    {
        let mut c = source.borrow_mut();
        c.user = Some(Username::new_coerce(&cmd.args[0]));
        c.realname = Some(cmd.args[3].clone());
        if c.can_register()
        {
            self.action(CommandAction::RegisterClient(cmd.connection.id()))?;
        }
        Ok(())
    }
});
