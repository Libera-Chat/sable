use super::*;

#[command_handler("NOTICE")]
async fn handle_notice(
    server: &ClientServer,
    source: UserSource<'_>,
    cmd: &dyn Command,
    target: TargetParameter<'_>,
    msg: &str,
) -> CommandResult {
    if msg.len() == 0 {
        return numeric_error!(NoTextToSend);
    }

    if let Some(user) = target.user() {
        if user.is_alias_user().is_some() {
            // This is a notice which doesn't expect a response; drop it
            return Ok(());
        }
    }
    if let Some(channel) = target.channel() {
        server.policy().can_send(&source, &channel, msg)?;
    }

    let details = event::details::NewMessage {
        source: source.id(),
        target: target.object_id(),
        message_type: state::MessageType::Notice,
        text: msg.to_owned(),
    };
    cmd.new_event_with_response(server.ids().next_message(), details)
        .await;
    Ok(())
}
