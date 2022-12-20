use super::*;
use event::*;

#[command_handler("NICK")]
fn handle_nick(server: &ClientServer, cmd: &ClientCommand, source: CommandSource, new_nick: Nickname) -> CommandResult
{
    match source
    {
        CommandSource::User(user) => handle_user(server, cmd, user, new_nick),
        CommandSource::PreClient(pc) => handle_preclient(server, cmd, &pc, new_nick)
    }
}

fn handle_preclient(server: &ClientServer, cmd: &ClientCommand, source: &PreClient, nick: Nickname) -> CommandResult
{
    if server.network().nick_binding(&nick).is_ok()
    {
        numeric_error!(NicknameInUse, &nick)
    }
    else
    {
        source.nick.set(nick).ok(); // Ignore the result; if the preclient already has a nick then we silently ignore
                                    // a new one
        if source.can_register()
        {
            server.add_action(CommandAction::RegisterClient(cmd.connection.id()));
        }

        Ok(())
    }
}

fn handle_user(server: &ClientServer, cmd: &ClientCommand, source: wrapper::User, nick: Nickname) -> CommandResult
{
    let detail = details::BindNickname{ user: source.id() };

    if server.network().nick_binding(&nick).is_ok()
    {
        numeric_error!(NicknameInUse, &nick)
    }
    else
    {
        server.add_action(CommandAction::state_change(NicknameId::new(nick), detail));

        Ok(())
    }
}
