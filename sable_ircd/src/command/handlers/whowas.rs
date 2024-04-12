use super::*;

const DEFAULT_COUNT: usize = 8; // Arbitrary value, that happens to match the capacity of
                                // historic_nick_users

#[command_handler("WHOWAS")]
/// Syntax: WHOIS <target> [<count>]
fn whowas_handler(
    network: &Network,
    response: &dyn CommandResponse,
    source: UserSource,
    server: &ClientServer,
    target: Nickname,
    count: Option<u32>,
) -> CommandResult {
    // "If given, <count> SHOULD be a positive number. Otherwise, a full search is done."
    let count = match count {
        None | Some(0) => DEFAULT_COUNT,
        Some(count) => count.try_into().unwrap_or(usize::MAX),
    };
    let historic_users: Vec<_> = network
        .historic_users_by_nick(&target)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .take(count)
        .collect();

    if historic_users.is_empty() {
        response.numeric(make_numeric!(WasNoSuchNick, &target));
    } else {
        for historic_user in historic_users {
            let user: sable_network::network::wrapper::User<'_> =
                wrapper::ObjectWrapper::wrap(network, &historic_user.user);
            response.numeric(make_numeric!(WhowasUser, &historic_user));

            if let Ok(Some(account)) = user.account() {
                response.numeric(make_numeric!(WhoisAccount, &target, &account.name()));
            }

            if server.policy().can_see_connection_info(&source, &user) {
                for conn in user.connections() {
                    response.numeric(make_numeric!(WhoisServer, &user, &conn.server()?));
                    response.numeric(make_numeric!(WhoisHost, &user, conn.hostname(), conn.ip()));
                }
            }
        }
    }

    response.numeric(make_numeric!(EndOfWhowas, &target));
    Ok(())
}
