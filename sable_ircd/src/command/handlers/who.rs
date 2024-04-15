use super::*;
use crate::capability::ClientCapability;
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

                send_who_reply(response, &member.user()?, Some(&channel), Some(&member));
            }
        }
    } else if let Ok(nick) = Nickname::from_str(target) {
        if let Ok(user) = network.user_by_nick(&nick) {
            send_who_reply(
                response, &user, None, // channel
                None, // membership
            );
        }
    }

    // If nick/channel is not found, EndOfWho should be the only numeric we send
    response.numeric(make_numeric!(EndOfWho, target));

    Ok(())
}

fn send_who_reply(
    response: &dyn CommandResponse,
    target: &wrapper::User,
    channel: Option<&wrapper::Channel>,
    membership: Option<&wrapper::Membership>,
) {
    let chname = channel.map(|c| c.name().value() as &str).unwrap_or("*");
    let away_letter = match target.away_reason() {
        None => 'H',    // Here
        Some(_) => 'G', // Gone
    };
    let status = if response.capabilities().has(ClientCapability::MultiPrefix) {
        format!(
            "{}{}",
            away_letter,
            membership
                .map(|m| m.permissions().to_prefixes())
                .unwrap_or_else(|| "".to_string())
        )
    } else {
        format!(
            "{}{}",
            away_letter,
            membership
                .and_then(|m| m.permissions().to_highest_prefix())
                .as_ref()
                .map(char::to_string)
                .unwrap_or_else(|| "".to_string())
        )
    };
    response.numeric(make_numeric!(WhoReply, chname, target, &status, 0))
}
