use super::*;

#[command_handler("INVITE")]
fn handle_invite(server: &ClientServer, source: UserSource,
                 target: wrapper::User, channel: wrapper::Channel) -> CommandResult
{
    let source = source.deref();

    server.policy().can_invite(source, &channel, &target)?;

    let invite_id = InviteId::new(target.id(), channel.id());

    let event = event::details::ChannelInvite {
        source: source.id()
    };

    server.add_action(CommandAction::state_change(invite_id, event));

    Ok(())
}
