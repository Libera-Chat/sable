use super::*;
use CommandAction::StateChange;


command_handler!("JOIN" => JoinHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let chname = ChannelName::new(cmd.args[0].clone())?;
        let channel_id = match self.server.network().channel_by_name(&chname) {
            Ok(channel) => channel.id(),
            Err(_) => {
                let newmode_details = event::NewChannelMode { mode: ChannelModeSet::default() };
                let cmode_id = self.server.next_cmode_id();
                let newmode_event = self.server.create_event(cmode_id, newmode_details);
                self.action(StateChange(newmode_event))?;

                let details = event::NewChannel { name: chname.clone(), mode: cmode_id };
                let channel_id = self.server.next_channel_id();
                let event = self.server.create_event(channel_id, details);
                self.action(StateChange(event))?;
                channel_id
            }
        };
        let details = event::ChannelJoin {
            user: source.id(),
            channel: channel_id,
        };
        let membership_id = MembershipId::new(source.id(), channel_id);
        let event = self.server.create_event(membership_id, details);
        self.action(StateChange(event))?;
        Ok(())
    }
});