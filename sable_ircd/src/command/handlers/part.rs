use super::*;

#[command_handler("PART")]
fn handle_part(server: &ClientServer, net: &Network, source: UserSource,
               channel: wrapper::Channel, msg: &str) -> CommandResult
{
    let membership_id = MembershipId::new(source.id(), channel.id());
    if net.membership(membership_id).is_ok()
    {
        let details = event::ChannelPart{ message: msg.to_owned() };
        server.add_action(CommandAction::state_change(membership_id, details));
    } else {
        return numeric_error!(NotOnChannel, channel.name());
    }
    Ok(())
}
