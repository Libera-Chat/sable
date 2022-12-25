use super::*;
use event::*;

#[command_handler("NICK")]
fn handle_nick(server: &ClientServer, net: &Network, cmd: &dyn Command, source: CommandSource, new_nick: Nickname) -> CommandResult
{
    match source
    {
        CommandSource::User(user) => handle_user(server, net, user, new_nick),
        CommandSource::PreClient(pc) => handle_preclient(server, net, cmd, &pc, new_nick)
    }
}

fn handle_preclient(server: &ClientServer, net: &Network, cmd: &dyn Command, source: &PreClient, nick: Nickname) -> CommandResult
{
    if net.nick_binding(&nick).is_ok()
    {
        numeric_error!(NicknameInUse, &nick)
    }
    else
    {
        source.nick.set(nick).ok(); // Ignore the result; if the preclient already has a nick then we silently ignore
                                    // a new one
        if source.can_register()
        {
            server.add_action(CommandAction::RegisterClient(cmd.connection()));
        }

        Ok(())
    }
}

fn handle_user(server: &ClientServer, net: &Network, source: wrapper::User, nick: Nickname) -> CommandResult
{
    let detail = details::BindNickname{ user: source.id() };

    if net.nick_binding(&nick).is_ok()
    {
        numeric_error!(NicknameInUse, &nick)
    }
    else
    {
        server.add_action(CommandAction::state_change(NicknameId::new(nick), detail));

        Ok(())
    }
}
