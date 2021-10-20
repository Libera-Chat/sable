use super::*;

command_handler!("NICK", NickHandler);

impl CommandHandler for NickHandler
{
    fn min_parameters(&self) -> usize { 1 }

    fn handle_preclient(&self, server: &Server, source: &RefCell<PreClient>, cmd: &ClientCommand, proc: &mut CommandProcessor) -> CommandResult
    {
        let nick = cmd.args[0].clone();
        if server.network().user_by_nick(&nick).is_ok()
        {
            cmd.connection.send(&numeric::NicknameInUse::new(server, &*source.borrow(), &nick))?;
        } else {
            let mut c = source.borrow_mut();
            c.nick = Some(nick);
            if c.can_register()
            {
                proc.action(CommandAction::RegisterClient(cmd.connection.id())).translate(cmd)?;
            }
        }
        Ok(())
    }

    fn handle_user(&self, _server: &Server, _source: &wrapper::User, _cmd: &ClientCommand, _proc: &mut CommandProcessor) -> CommandResult
    {
        panic!("not implemented");
    }
}