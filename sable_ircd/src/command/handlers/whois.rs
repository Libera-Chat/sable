use super::*;

#[command_handler("WHOIS")]
/// Syntax: WHOIS [&lt;server|target&gt;] &lt;target&gt;
fn whois_handler(
    response: &dyn CommandResponse,
    source: UserSource,
    server: &ClientServer,
    mut args: ArgList,
) -> CommandResult {
    let target: wrapper::User = match args.len() {
        0 => return Err(CommandError::NotEnoughParameters),
        1 => args.next()?,
        _ => {
            args.next::<&str>()?; // Remote whois is not implemented yet, ignore server name
            args.next()?
        }
    };

    response.numeric(make_numeric!(WhoisUser, &target));

    if let Ok(Some(account)) = target.account() {
        response.numeric(make_numeric!(WhoisAccount, &target, &account.name()));
    }

    if let Some(away_reason) = target.away_reason() {
        response.numeric(make_numeric!(Away, &target, away_reason));
    }

    if server.policy().can_see_connection_info(&source, &target) {
        for conn in target.connections() {
            response.numeric(make_numeric!(WhoisServer, &target, &conn.server()?));
            response.numeric(make_numeric!(
                WhoisHost,
                &target,
                conn.hostname(),
                conn.ip()
            ));
        }
    }

    response.numeric(make_numeric!(EndOfWhois, &target));
    Ok(())
}
