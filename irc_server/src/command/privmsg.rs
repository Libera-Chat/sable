use super::*;

command_handler!("PRIVMSG" => PrivmsgHandler {
    fn min_parameters(&self) -> usize { 2 }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let target_name = &cmd.args[0];
        let msg = cmd.args[1].clone();
        let target_id =
            if let Ok(chname) = ChannelName::from_str(target_name)
            {
                let channel = self.server.network().channel_by_name(&chname)?;
                self.server.policy().can_send(source, &channel, &msg)?;
                ObjectId::Channel(channel.id())
            }
            else if let Ok(nick) = Nickname::from_str(target_name)
            {
                ObjectId::User(self.server.network().user_by_nick(&nick)?.id())
            }
            else
            {
                return Err(numeric::NoSuchTarget::new(&target_name).into());
            };

        let details = event::details::NewMessage {
            source: source.id(),
            target: target_id,
            text: msg,
        };
        self.action(CommandAction::state_change(self.server.next_message_id(), details))?;
        Ok(())
    }
});