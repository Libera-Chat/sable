use super::*;
use crate::utils::make_numeric;

#[command_handler("WHO")]
fn handle_who(
    server: &ClientServer,
    network: &Network,
    response: &dyn CommandResponse,
    source: UserSource,
    target: &str,
) -> CommandResult {
    if let Ok(chname) = ChannelName::from_str(target) {
        if let Ok(channel) = network.channel_by_name(&chname) {
            for member in channel.members() {
                if server
                    .policy()
                    .can_see_user_on_channel(&source, &member)
                    .is_err()
                {
                    continue;
                }

                response.numeric(make_who_reply(
                    &member.user()?,
                    Some(&channel),
                    Some(&member),
                    &member.user()?.server()?,
                ));
            }
        }
    } else if let Ok(nick) = Nickname::from_str(target) {
        if let Ok(user) = network.user_by_nick(&nick) {
            response.numeric(make_who_reply(
                &user,
                None, // channel
                None, // membership
                &user.server()?,
            ));
        }
    }

    // If nick/channel is not found, EndOfWho should be the only numeric we send
    response.numeric(make_numeric!(EndOfWho, target));

    Ok(())
}

fn make_who_reply(
    target: &wrapper::User,
    channel: Option<&wrapper::Channel>,
    membership: Option<&wrapper::Membership>,
    server: &wrapper::Server,
) -> UntargetedNumeric {
    let chname = channel.map(|c| c.name().value() as &str).unwrap_or("*");
    let away_letter = match target.away_reason() {
        None => 'H',    // Here
        Some(_) => 'G', // Gone
    };
    let status = format!(
        "{}{}",
        away_letter,
        membership
            .map(|m| m.permissions().to_prefixes())
            .unwrap_or_else(|| "".to_string())
    );
    make_numeric!(WhoReply, chname, target, server, &status, 0)
}
