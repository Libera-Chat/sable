use super::*;

#[command_handler("NOTICE")]
fn handle_notice(
    server: &ClientServer,
    source: UserSource,
    target: TargetParameter,
    msg: &str,
) -> CommandResult {
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
    server.add_action(CommandAction::state_change(
        server.ids().next_message(),
        details,
    ));
    Ok(())
}
