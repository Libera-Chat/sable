use super::*;


command_handler!("JOIN" => JoinHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let empty_str = String::new();
        let names = cmd.args[0].split(',');
        let mut keys = cmd.args.get(1).unwrap_or(&empty_str).split(',');

        for name in names {
            let chname = ChannelName::from_str(name)?;
            let key = keys.next().map(|s| ChannelKey::new_coerce(&s));

            let (channel_id, permissions) = match self.server.network().channel_by_name(&chname) {
                Ok(channel) => {
                    self.server.policy().can_join(source, &channel, key)?;

                    (channel.id(), MembershipFlagSet::new())
                },
                Err(_) => {
                    let newmode_details = event::NewChannelMode { mode: ChannelModeSet::default() };
                    let cmode_id = self.server.ids().next_channel_mode();
                    self.action(CommandAction::state_change(cmode_id, newmode_details))?;

                    let details = event::NewChannel { name: chname.clone(), mode: cmode_id };
                    let channel_id = self.server.ids().next_channel();
                    self.action(CommandAction::state_change(channel_id, details))?;
                    (channel_id, MembershipFlagFlag::Op.into())
                }
            };
            let details = event::ChannelJoin {
                user: source.id(),
                channel: channel_id,
                permissions: permissions,
            };
            let membership_id = MembershipId::new(source.id(), channel_id);
            self.action(CommandAction::state_change(membership_id, details))?;
        }
        Ok(())
    }
});