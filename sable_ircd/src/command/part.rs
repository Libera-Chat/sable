use super::*;

command_handler!("PART" => PartHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let chname = ChannelName::from_str(&cmd.args[0])?;
        let net = self.server.network();
        let channel = net.channel_by_name(&chname)?;
        let msg = cmd.args.get(1).unwrap_or(&"".to_string()).clone();

        let membership_id = MembershipId::new(source.id(), channel.id());
        if self.server.network().membership(membership_id).is_ok()
        {
            let details = event::ChannelPart{ message: msg };
            self.action(CommandAction::state_change(membership_id, details))?;
        } else {
            return numeric_error!(NotOnChannel, &chname);
        }
        Ok(())
    }
});