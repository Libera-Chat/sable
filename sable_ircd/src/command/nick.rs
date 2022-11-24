use super::*;
use event::*;

command_handler!("NICK" => NickHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_preclient(&mut self, source: &PreClient, cmd: &ClientCommand) -> CommandResult
    {
        let nick = Nickname::from_str(&cmd.args[0])?;
        if self.server.network().nick_binding(&nick).is_ok()
        {
            cmd.connection.send(&numeric::NicknameInUse::new_for(&self.server, &UnknownTarget, &nick));
        }
        else
        {
            source.nick.set(nick).ok(); // Ignore the result; if the preclient already has a nick then we silently ignore
                                        // a new one
            if source.can_register()
            {
                self.action(CommandAction::RegisterClient(cmd.connection.id()))?;
            }
        }
        Ok(())
    }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let newnick = Nickname::from_str(&cmd.args[0])?;
        let detail = details::BindNickname{ user: source.id() };

        if self.server.network().nick_binding(&newnick).is_ok()
        {
            return numeric_error!(NicknameInUse, &newnick);
        }

        self.action(CommandAction::state_change(NicknameId::new(newnick), detail))?;

        Ok(())
    }
});