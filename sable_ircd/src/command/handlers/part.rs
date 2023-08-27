use super::*;

#[command_handler("PART")]
async fn handle_part(
    cmd: &dyn Command,
    net: &Network,
    source: UserSource<'_>,
    channel: wrapper::Channel<'_>,
    msg: Option<&str>,
) -> CommandResult {
    let membership_id = MembershipId::new(source.id(), channel.id());
    if net.membership(membership_id).is_ok() {
        let details = event::ChannelPart {
            message: msg.unwrap_or(source.nick().as_ref()).to_owned(),
        };
        cmd.new_event_with_response(membership_id, details).await;
    } else {
        return numeric_error!(NotOnChannel, channel.name());
    }
    Ok(())
}
