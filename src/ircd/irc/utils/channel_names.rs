use crate::ircd::*;
use irc::*;
use irc::client::*;
use wrapper::*;

pub fn send_channel_names(server: &Server, to: &ClientConnection, channel: &Channel) -> HandleResult
{
    let names = channel.members()
                       .map(|m| { Ok((m.user()?.nick().clone(), m.permissions())) })
                       .collect::<Result<Vec<(Nickname,ChannelPermissionSet)>, LookupError>>()?;

    let mut lines = Vec::new();
    let mut current_line = String::new();
    const CONTENT_LEN:usize = 300;

    for (n,p) in names
    {
        let p = p.to_prefixes();
        let n = n.to_string();
        if current_line.len() + n.len() + 1 > CONTENT_LEN
        {
            lines.push(current_line);
            current_line = String::new();
        }
        current_line += &format!("{}{} ", p, n);
    }
    lines.push(current_line);

    let user = server.network().user(to.user_id.ok_or(HandlerError::InternalError("Sending to non-user".to_string()))?)?;

    for line in lines
    {
        to.send(&numeric::NamesReply::new_for(server, &user, '*', &channel, &line))?;
    }
    to.send(&numeric::EndOfNames::new_for(server, &user, &channel))?;
    Ok(())
}