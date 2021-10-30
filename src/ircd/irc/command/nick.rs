use super::*;
use event::*;

command_handler!("NICK" => NickHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_preclient(&mut self, source: &RefCell<PreClient>, cmd: &ClientCommand) -> CommandResult
    {
        let nick = Nickname::new(cmd.args[0].clone())?;
        if self.server.network().user_by_nick(&nick).is_ok()
        {
            cmd.connection.send(&numeric::NicknameInUse::new_for(self.server, &*source.borrow(), &nick));
        }
        else
        {
            let mut c = source.borrow_mut();
            c.nick = Some(nick);
            if c.can_register()
            {
                self.action(CommandAction::RegisterClient(cmd.connection.id()))?;
            }
        }
        Ok(())
    }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let newnick = Nickname::new(cmd.args[0].clone())?;
        let detail = details::UserNickChange{ new_nick: newnick };

        self.server.network().validate(source.id().into(), &detail.clone().into())?;

        self.action(CommandAction::state_change(source.id(), detail))?;

        Ok(())
    }
});