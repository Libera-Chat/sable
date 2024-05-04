use super::*;
use sable_network::network::config::AliasUser;

#[command_handler("PRIVMSG")]
async fn handle_privmsg(
    server: &ClientServer,
    response: &dyn CommandResponse,
    source: UserSource<'_>,
    cmd: &dyn Command,
    target: TargetParameter<'_>,
    msg: &str,
) -> CommandResult {
    if msg.is_empty() {
        return numeric_error!(NoTextToSend);
    }

    match &target {
        TargetParameter::User(user) => {
            if let Some(AliasUser { command_alias, .. }) = user.is_alias_user() {
                return super::services::dispatch_alias_command(cmd, user, command_alias, msg)
                    .await;
            }

            if let Some(away_reason) = user.away_reason() {
                response.numeric(make_numeric!(Away, &user, away_reason));
            }
        }
        TargetParameter::Channel(channel) => {
            server.policy().can_send(&source, channel, msg)?;
        }
    }

    let details = event::details::NewMessage {
        source: source.id(),
        target: target.object_id(),
        message_type: state::MessageType::Privmsg,
        text: msg.to_owned(),
    };
    cmd.new_event_with_response(server.ids().next_message(), details)
        .await;
    Ok(())
}
