use super::*;
use CommandAction::StateChange;


command_handler!("JOIN", JoinHandler);

impl CommandHandler for JoinHandler
{
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&self, server: &Server, source: &wrapper::User, cmd: &ClientCommand, proc: &mut CommandProcessor) -> CommandResult
    {
        let chname = ChannelName::new(cmd.args[0].clone()).translate(cmd)?;
        let channel_id = match server.network().channel_by_name(&chname) {
            Ok(channel) => channel.id(),
            Err(_) => {
                let details = event::NewChannel { name: chname.clone() };
                let channel_id = server.next_channel_id();
                let event = server.create_event(channel_id, details);
                proc.action(StateChange(event)).translate(cmd)?;
                channel_id
            }
        };
        let details = event::ChannelJoin {
            user: source.id(),
            channel: channel_id,
        };
        let membership_id = MembershipId::new(source.id(), channel_id);
        let event = server.create_event(membership_id, details);
        proc.action(StateChange(event)).translate(cmd)?;
        Ok(())
    }
}