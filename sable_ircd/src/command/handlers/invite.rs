use super::*;

#[command_handler("INVITE")]
fn handle_invite(
    server: &ClientServer,
    source: UserSource,
    response: &dyn CommandResponse,
    target: wrapper::User,
    channel: wrapper::Channel,
) -> CommandResult {
    if target.is_in_channel(channel.id()).is_some() {
        return numeric_error!(UserOnChannel, &target, &channel);
    }

    let source = source.deref();

    server.policy().can_invite(source, &channel, &target)?;

    let invite_id = InviteId::new(target.id(), channel.id());

    let event = event::details::ChannelInvite {
        source: source.id(),
    };

    server.add_action(CommandAction::state_change(invite_id, event));

    response.numeric(make_numeric!(Inviting, &target, &channel));

    Ok(())
}
