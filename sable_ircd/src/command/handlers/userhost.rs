use super::*;
use itertools::Itertools;

/// "The USERHOST command takes up to five nicknames, each a separate parameters."
/// -- https://modern.ircdocs.horse/#userhost-message
const MAX_TARGETS: usize = 5;

fn away_char(user: &wrapper::User) -> char {
    if user.away_reason().is_some() {
        '-'
    } else {
        '+'
    }
}

#[command_handler("USERHOST")]
/// Syntax: USERHOST <nickname>{ <nickname>}
fn userhost_handler(
    response: &dyn CommandResponse,
    network: &Network,
    args: ArgList,
) -> CommandResult {
    if args.is_empty() {
        Err(CommandError::NotEnoughParameters)
    } else {
        let reply = args
            .iter()
            .take(MAX_TARGETS)
            .flat_map(Nickname::convert)
            .flat_map(|nick| network.user_by_nick(&nick))
            .map(|user| {
                format!(
                    "{}={}{}@{}",
                    user.nick(),
                    away_char(&user),
                    user.user(),
                    user.visible_host()
                )
            })
            .join(" ");

        response.numeric(make_numeric!(Userhost, &reply));
        Ok(())
    }
}
