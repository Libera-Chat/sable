use super::*;

#[command_handler("KICK")]
async fn handle_kick(
    server: &ClientServer,
    cmd: &dyn Command,
    net: &Network,
    source: UserSource<'_>,
    channel: wrapper::Channel<'_>,
    target: wrapper::User<'_>,
    message: Option<&str>,
) -> CommandResult {
    let source_membership_id = MembershipId::new(source.id(), channel.id());
    if net.membership(source_membership_id).is_err() {
        return numeric_error!(NotOnChannel, channel.name());
    }

    let target_membership_id = MembershipId::new(target.id(), channel.id());
    if net.membership(target_membership_id).is_err() {
        return numeric_error!(UserNotOnChannel, &target, &channel);
    }

    let message = message.unwrap_or(source.nick().as_ref()).to_owned();

    server
        .policy()
        .can_kick(&source, &channel, &target, &message)?;

    let details = event::ChannelKick {
        source: source.id(),
        message,
    };
    cmd.new_event_with_response(target_membership_id, details)
        .await;

    Ok(())
}
