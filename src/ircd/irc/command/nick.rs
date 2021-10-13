use super::*;

command_handler!("NICK", NickHandler);

impl CommandHandler for NickHandler
{
    fn min_parameters(&self) -> usize { 1 }

    fn handle_preclient(&self, _server: &Server, source: &RefCell<PreClient>, cmd: &ClientCommand, actions: &mut Vec<CommandAction>) -> CommandResult
    {
        let mut c = source.borrow_mut();
        c.nick = Some(cmd.args[0].clone());
        if c.can_register()
        {
            actions.push(CommandAction::RegisterClient(cmd.connection.id()));
        }
        Ok(())
    }

    fn handle_user(&self, _server: &Server, _source: &wrapper::User, _cmd: &ClientCommand, _actions: &mut Vec<CommandAction>) -> CommandResult
    {
        panic!("not implemented");
    }
}