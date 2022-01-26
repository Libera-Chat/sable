use super::*;

command_handler!("CAP" => UserHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_preclient(&mut self, source: &RefCell<PreClient>, cmd: &ClientCommand) -> CommandResult
    {
        let mut c = source.borrow_mut();
        if cmd.args[0] == "LS"
        {
            c.cap_in_progress = true;
            cmd.connection.send(&message::Cap::new(self.server, &*c, "LS", ""));
        }
        else if cmd.args[0] == "END"
        {
            c.cap_in_progress = false;
            if c.can_register()
            {
                self.action(CommandAction::RegisterClient(cmd.connection.id()))?;
            }
        }
        
        Ok(())
    }
});
