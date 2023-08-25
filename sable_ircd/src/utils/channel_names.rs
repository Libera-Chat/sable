use crate::errors::*;
use crate::messages::numeric;
use crate::messages::*;
use crate::*;
use sable_network::prelude::*;

use std::fmt::Write;

pub fn send_channel_names(
    server: &ClientServer,
    to: impl MessageSink,
    to_user: &wrapper::User,
    channel: &wrapper::Channel,
) -> HandleResult {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    const CONTENT_LEN: usize = 300;

    let pub_or_secret = if channel.mode().has_mode(ChannelModeFlag::Secret) {
        '@'
    } else {
        '='
    };

    let user_is_on_chan = to_user.is_in_channel(channel.id()).is_some();

    for member in channel.members() {
        if !user_is_on_chan
            && server
                .policy()
                .can_see_user_on_channel(to_user, &member)
                .is_err()
        {
            continue;
        }

        let p = member.permissions().to_prefixes();
        let n = member.user()?.nick().to_string();
        if current_line.len() + n.len() + 2 > CONTENT_LEN {
            lines.push(current_line);
            current_line = String::new();
        }
        current_line.write_fmt(format_args!("{}{} ", p, n))?;
    }
    lines.push(current_line);

    for line in lines {
        to.send(
            numeric::NamesReply::new(pub_or_secret, channel, &line).format_for(server, to_user),
        );
    }
    to.send(numeric::EndOfNames::new(channel).format_for(server, to_user));
    Ok(())
}
