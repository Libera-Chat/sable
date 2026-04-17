use super::*;

#[command_handler("LIST")]
fn handle_list(
    response: &dyn CommandResponse,
    source: UserSource,
    net: &Network,
    target: Option<&str>,
) -> CommandResult {
    match target {
        Some(channel_str) => {
            // Try to list a specific channel
            if let Ok(chname) = ChannelName::from_str(channel_str) {
                if let Ok(channel) = net.channel_by_name(&chname) {
                    let topic = channel.topic().map(|t| t.text().to_owned()).unwrap_or_default();
                    let count = channel.members().count();
                    response.numeric(make_numeric!(ListReply, &channel, count, topic.as_str()));
                }
            }
        }
        None => {
            // List all channels
            for channel in net.channels() {
                // Only list channels that aren't secret (+s) or that the user is a member of
                let is_secret = channel.mode().has_mode(ChannelModeFlag::Secret);
                let is_member = channel.has_member(source.id()).is_some();

                if !is_secret || is_member {
                    let topic = channel.topic().map(|t| t.text().to_owned()).unwrap_or_default();
                    let count = channel.members().count();
                    response.numeric(make_numeric!(ListReply, &channel, count, topic.as_str()));
                }
            }
        }
    }

    response.numeric(make_numeric!(EndOfList));
    Ok(())
}
