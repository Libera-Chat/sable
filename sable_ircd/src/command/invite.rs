use super::*;

command_handler!("INVITE" => InviteHandler {
    fn min_parameters(&self) -> usize { 2 }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let target_nick = Nickname::from_str(&cmd.args[0])?;
        let chname = ChannelName::from_str(&cmd.args[1])?;

        let net = self.server.network();
        let channel = net.channel_by_name(&chname)?;
        let target = net.user_by_nick(&target_nick)?;

        self.server.policy().can_invite(source, &channel, &target)?;

        let invite_id = InviteId::new(target.id(), channel.id());

        let event = event::details::ChannelInvite {
            source: source.id()
        };

        self.action(CommandAction::state_change(invite_id, event))?;

        Ok(())
    }
});