use super::*;

#[command_handler("WHOIS")]
/// Syntax: WHOIS [&lt;server|target&gt;] &lt;target&gt;
fn whois_handler(
    command: &dyn Command,
    response: &dyn CommandResponse,
    source: UserSource,
    server: &ClientServer,
    net: &Network,
    mut args: ArgList,
) -> CommandResult {
    let target_str: &str = match args.len() {
        0 => return Err(CommandError::NotEnoughParameters),
        1 => args.next()?,
        _ => {
            args.next::<&str>()?; // Remote whois is not implemented yet, ignore server name
            args.next()?
        }
    };

    // We need to handle the no-such-user and invalid-nick cases manually because we have to send
    // EndOfWhois even when the arguments are invalid
    let target_nick = match Nickname::try_from(target_str) {
        Ok(nick) => nick,
        Err(err) => {
            command.notify_error(err.into());
            response.numeric(make_numeric!(EndOfWhois, target_str));
            return Ok(());
        }
    };
    let target = match net.user_by_nick(&target_nick) {
        Ok(user) => user,
        Err(err) => {
            command.notify_error(err.into());
            response.numeric(make_numeric!(EndOfWhois, target_str));
            return Ok(());
        }
    };

    response.numeric(make_numeric!(WhoisUser, &target));

    if let Ok(Some(account)) = target.account() {
        response.numeric(make_numeric!(WhoisAccount, &target.nick(), &account.name()));
    }

    if let Some(away_reason) = target.away_reason() {
        response.numeric(make_numeric!(Away, &target, away_reason));
    }

    if server.policy().can_see_connection_info(&source, &target) {
        for conn in target.connections() {
            if let Ok(server) = conn.server() {
                response.numeric(make_numeric!(WhoisServer, &target, &server));
            }
            response.numeric(make_numeric!(
                WhoisHost,
                &target,
                conn.hostname(),
                conn.ip()
            ));
        }
    }

    response.numeric(make_numeric!(EndOfWhois, &target_str));
    Ok(())
}
