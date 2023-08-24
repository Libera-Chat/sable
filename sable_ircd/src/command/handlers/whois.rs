use super::*;

#[sable_macros::command_handler("WHOIS")]
fn whois_handler(
    response: &dyn CommandResponse,
    _source: UserSource,
    target: wrapper::User,
) -> CommandResult {
    response.numeric(make_numeric!(WhoisUser, &target));

    if let Ok(server) = target.server() {
        response.numeric(make_numeric!(WhoisServer, &target, &server));
    }

    if let Ok(Some(account)) = target.account() {
        response.numeric(make_numeric!(WhoisAccount, &target, &account.name()));
    }

    response.numeric(make_numeric!(EndOfWhois, &target));
    Ok(())
}
