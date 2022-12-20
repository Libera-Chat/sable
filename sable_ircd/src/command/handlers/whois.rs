use super::*;

#[sable_macros::command_handler("WHOIS")]
fn whois_handler(cmd: &ClientCommand, _source: UserSource, target: wrapper::User) -> CommandResult
{
    cmd.numeric(make_numeric!(WhoisUser, &target));
    cmd.numeric(make_numeric!(WhoisServer, &target, &target.server()?));

    if let Ok(Some(account)) = target.account()
    {
        cmd.numeric(make_numeric!(WhoisAccount, &target, &account.name()));
    }

    cmd.numeric(make_numeric!(EndOfWhois, &target));
    Ok(())
}
