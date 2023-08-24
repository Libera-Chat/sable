use super::*;

#[command_handler("PRIVMSG")]
async fn handle_privmsg(
    server: &ClientServer,
    source: UserSource<'_>,
    cmd: &dyn Command,
    target: TargetParameter<'_>,
    msg: &str,
) -> CommandResult {
    if let Some(user) = target.user() {
        if let Some(alias) = user.is_alias_user() {
            return super::services::dispatch_alias_command(cmd, &user, &alias.command_alias, msg)
                .await;
        }
    }

    if let Some(channel) = target.channel() {
        server.policy().can_send(&source, &channel, msg)?;
    }

    let details = event::details::NewMessage {
        source: source.id(),
        target: target.object_id(),
        message_type: state::MessageType::Privmsg,
        text: msg.to_owned(),
    };
    server.add_action(CommandAction::state_change(
        server.ids().next_message(),
        details,
    ));
    Ok(())
}
