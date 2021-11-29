use irc_network::*;
use irc_network::wrapper::Channel;
use crate::*;
use crate::client::*;
use crate::errors::*;

pub fn send_channel_names(server: &Server, to: &ClientConnection, channel: &Channel) -> HandleResult
{
    let user = server.network().user(to.user_id.ok_or(HandlerError::InternalError("Sending to non-user".to_string()))?)?;

    let mut lines = Vec::new();
    let mut current_line = String::new();
    const CONTENT_LEN:usize = 300;

    let pub_or_secret = if channel.mode()?.has_mode(ChannelModeFlag::Secret) {
        '@'
    } else {
        '='
    };

    for member in channel.members()
    {
        if server.policy().can_see_user_on_channel(&user, &member).is_err()
        {
            continue;
        }

        let p = member.permissions().to_prefixes();
        let n = member.user()?.nick().to_string();
        if current_line.len() + n.len() + 1 > CONTENT_LEN
        {
            lines.push(current_line);
            current_line = String::new();
        }
        current_line += &format!("{}{} ", p, n);
    }
    lines.push(current_line);

    for line in lines
    {
        to.send(&numeric::NamesReply::new_for(server, &user, pub_or_secret, &channel, &line));
    }
    to.send(&numeric::EndOfNames::new_for(server, &user, &channel));
    Ok(())
}