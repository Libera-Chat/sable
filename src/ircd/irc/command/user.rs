use super::*;

command_handler!("USER", UserHandler);

impl CommandHandler for UserHandler
{
    fn min_parameters(&self) -> usize { 4 }

    fn handle_preclient(&self, _server: &Server, source: &RefCell<PreClient>, cmd: &ClientCommand, actions: &mut Vec<CommandAction>) -> Result<(), CommandError>
    {
        let mut c = source.borrow_mut();
        c.user = Some(cmd.args[0].clone());
        c.realname = Some(cmd.args[3].clone());
        if c.can_register()
        {
            actions.push(CommandAction::RegisterClient(cmd.connection.id()));
        }
        Ok(())
    }
}