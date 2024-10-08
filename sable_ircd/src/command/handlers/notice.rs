use super::*;

#[command_handler("NOTICE")]
async fn handle_notice(
    server: &ClientServer,
    source: UserSource<'_>,
    cmd: &dyn Command,
    target: Result<TargetParameter<'_>, &str>,
    msg: &str,
) -> CommandResult {
    let Ok(target) = target else {
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
    if msg.is_empty() {
        // Ditto
        return Ok(());
    }

    match &target {
        TargetParameter::User(user) => {
            if user.is_alias_user().is_some() {
                // This is a notice which doesn't expect a response; drop it
                return Ok(());
            }
        }
        TargetParameter::Channel(channel) => {
            if server.policy().can_send(&source, channel, msg).is_err() {
                // Silent error, see above
                return Ok(());
            }
        }
    }

    let details = event::details::NewMessage {
        source: source.id(),
        target: target.object_id(),
        message_type: state::MessageType::Notice,
        text: msg.to_owned(),
    };
    cmd.new_event_with_response(MessageId::new(Uuid7::new_now()), details)
        .await;
    Ok(())
}
