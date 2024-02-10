use super::*;
use event::*;

#[command_handler("NICK")]
async fn handle_nick(
    server: &ClientServer,
    net: &Network,
    cmd: &dyn Command,
    source: CommandSource<'_>,
    new_nick: Nickname,
) -> CommandResult {
    match source {
        CommandSource::User(user, _) => handle_user(net, cmd, user, new_nick).await,
        CommandSource::PreClient(pc) => handle_preclient(server, net, cmd, &pc, new_nick),
    }
}

fn handle_preclient(
    server: &ClientServer,
    net: &Network,
    cmd: &dyn Command,
    source: &PreClient,
    nick: Nickname,
) -> CommandResult {
    if net.user_by_nick(&nick).is_ok() {
        numeric_error!(NicknameInUse, &nick)
    } else {
        source.nick.set(nick).ok(); // Ignore the result; if the preclient already has a nick then we silently ignore
                                    // a new one
        if source.can_register() {
            server.add_action(CommandAction::RegisterClient(cmd.connection_id()));
        }

        Ok(())
    }
}

async fn handle_user(
    net: &Network,
    cmd: &dyn Command,
    source: wrapper::User<'_>,
    nick: Nickname,
) -> CommandResult {
    let detail = details::BindNickname { user: source.id() };

    match net.user_by_nick(&nick) {
        Ok(other_user) if other_user.id() == source.id() => {
            // The client is trying to change to a nickname case-equivalent to their
            // current one
            assert_eq!(nick, other_user.nick()); // Case-insensitive
            if nick.value() != other_user.nick().value() {
                // The nick is not exactly the same, issue the case change
                cmd.new_event_with_response(NicknameId::new(nick), detail)
                    .await;
            }
            Ok(())
        }
        Ok(_) => {
            numeric_error!(NicknameInUse, &nick)
        }
        Err(_) => {
            // Nickname is available
            cmd.new_event_with_response(NicknameId::new(nick), detail)
                .await;
            Ok(())
        }
    }
}
