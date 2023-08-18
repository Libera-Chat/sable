use super::*;
use crate::utils::make_numeric;

#[command_handler("WHO")]
fn handle_user(server: &ClientServer, cmd: &dyn Command, source: UserSource,
               channel: wrapper::Channel) -> CommandResult
{
    for member in channel.members()
    {
        if server.policy().can_see_user_on_channel(&source, &member).is_err()
        {
            continue;
        }

        cmd.numeric(make_who_reply(&member.user()?, Some(&channel), Some(&member), &member.user()?.server()?));
    }

    cmd.numeric(make_numeric!(EndOfWho, channel.name().value()));

    Ok(())
}

fn make_who_reply(target: &wrapper::User, channel: Option<&wrapper::Channel>,
                    membership: Option<&wrapper::Membership>, server: &wrapper::Server) -> UntargetedNumeric
{
    let chname = channel.map(|c| c.name().value() as &str).unwrap_or("*");
    let status = format!("H{}", membership.map(|m| m.permissions().to_prefixes()).unwrap_or_else(|| "".to_string()));
    make_numeric!(WhoReply, chname, target, server, &status, 0)
}
