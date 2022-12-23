use super::*;

#[command_handler("PRIVMSG")]
fn handle_privmsg(server: &ClientServer, source: UserSource,
    target: TargetParameter, msg: &str) -> CommandResult
{
    if let Some(channel) = target.channel()
    {
        server.policy().can_send(&source, &channel, msg)?;
    }

    let details = event::details::NewMessage {
        source: source.id(),
        target: target.object_id(),
        message_type: state::MessageType::Privmsg,
        text: msg.to_owned(),
    };
    server.add_action(CommandAction::state_change(server.ids().next_message(), details));
    Ok(())
}
