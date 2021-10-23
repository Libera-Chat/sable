use super::*;
use CommandAction::StateChange;

command_handler!("PART", PartHandler);

impl CommandHandler for PartHandler
{
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&self, server: &Server, source: &wrapper::User, cmd: &ClientCommand, proc: &mut CommandProcessor) -> CommandResult
    {
        let chname = ChannelName::new(cmd.args[0].clone())?;
        let channel = match server.network().channel_by_name(&chname) {
            Ok(c) => c,
            Err(_) => { return Err(numeric::NoSuchChannel::new(&chname).into()); }
        };
        let msg = cmd.args.get(1).unwrap_or(&"".to_string()).clone();

        let membership_id = MembershipId::new(source.id(), channel.id());
        if server.network().membership(membership_id).is_ok()
        {
            let details = event::ChannelPart{ message: msg };
            let event = server.create_event(membership_id, details);
            proc.action(StateChange(event))?;
        } else {
            return Err(numeric::NotOnChannel::new(&channel).into());
        }
        Ok(())
    }
}