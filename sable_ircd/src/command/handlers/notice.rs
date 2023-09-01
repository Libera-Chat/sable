use super::*;

#[command_handler("NOTICE")]
async fn handle_notice(
    server: &ClientServer,
    source: UserSource<'_>,
    cmd: &dyn Command,
    target: &str,
    msg: &str,
) -> CommandResult {
    let net = server.network();

    let target = if let Ok(chname) = ChannelName::from_str(target) {
        match net.channel_by_name(&chname).ok() {
            Some(channel) if server.policy().can_send(&source, &channel, msg).is_err() => None,
            Some(channel) => Some(TargetParameter::Channel(channel)),
            None => None,
        }
    } else if let Ok(nick) = Nickname::from_str(target) {
        match net.user_by_nick(&nick).ok() {
            Some(user) if user.is_alias_user().is_some() => {
                // This is a notice which doesn't expect a response; drop it
                return Ok(());
            }
            Some(user) => Some(TargetParameter::User(user)),
            None => None,
        }
    } else {
        None
    };

    let Some(target) = target else {
        /* No such target. However, the spec say we should not send an error:
         *
         * "automatic replies must never be
         * sent in response to a NOTICE message.  This rule applies to servers
         * too - they must not send any error reply back to the client on
         * receipt of a notice"
         * -- <https://tools.ietf.org/html/rfc1459#section-4.4.2>
         *
         * "automatic replies MUST NEVER be sent in response to a NOTICE message.
         * This rule applies to servers too - they MUST NOT send any error repl
         * back to the client on receipt of a notice."
         * -- <https://tools.ietf.org/html/rfc2812#section-3.3.2>
         *
         * "This rule also applies to servers – they must not send any error back
         * to the client on receipt of a NOTICE command"
         * -- https://modern.ircdocs.horse/#notice-message
         *
         * and most other servers agree with the specs, at least on non-existent
         * channels.
         */
        return Ok(());
    };

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
