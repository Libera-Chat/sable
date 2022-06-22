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
            let key = keys.next().map(ChannelKey::new_coerce);

            let (channel_id, permissions) = match self.server.network().channel_by_name(&chname) {
                Ok(channel) => {
                    self.server.policy().can_join(source, &channel, key)?;

                    (channel.id(), MembershipFlagSet::new())
                },
                Err(_) => {
                    let details = event::NewChannel {
                        name: chname,
                        mode: state::ChannelMode::new(ChannelModeSet::default()),
                    };
                    let channel_id = self.server.ids().next_channel();
                    self.action(CommandAction::state_change(channel_id, details))?;
                    (channel_id, MembershipFlagFlag::Op.into())
                }
            };
            let details = event::ChannelJoin {
                user: source.id(),
                channel: channel_id,
                permissions,
            };
            let membership_id = MembershipId::new(source.id(), channel_id);
            self.action(CommandAction::state_change(membership_id, details))?;
        }
        Ok(())
    }
});