use super::*;
use event::*;

#[command_handler("NICK")]
fn handle_nick(
    server: &ClientServer,
    net: &Network,
    cmd: &dyn Command,
    source: CommandSource,
    new_nick: Nickname,
) -> CommandResult {
    match source {
        CommandSource::User(user) => handle_user(server, net, user, new_nick),
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

fn handle_user(
    server: &ClientServer,
    net: &Network,
    source: wrapper::User,
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
                server.add_action(CommandAction::state_change(NicknameId::new(nick), detail));
            }
            Ok(())
        }
        Ok(_) => {
            numeric_error!(NicknameInUse, &nick)
        }
        Err(_) => {
            // Nickname is available
            server.add_action(CommandAction::state_change(NicknameId::new(nick), detail));
            Ok(())
        }
    }
}
