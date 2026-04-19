use super::*;

#[command_handler("JOIN")]
/// JOIN <#channel> [<key>]
///
/// The JOIN command allows you to enter a public chat area known as
/// a channel. Channels are prefixed with a '#'. More than one
/// channel may be specified, separated with commas (no spaces).
///
/// If the channel has a key set, the 2nd argument must be
/// given to enter. This allows channels to be password protected.
///
/// See also: PART, LIST
async fn handle_join(
    server: &ClientServer,
    net: &Network,
    cmd: &dyn Command,
    source: UserSource<'_>,
    channel_names: &str,
    keys: Option<&str>,
) -> CommandResult {
    let names = channel_names.split(',');
    let mut keys = match keys {
        None => Vec::new(),
        Some(keys) => keys.split(',').collect(),
    }
    .into_iter();

    for name in names {
        let chname = ChannelName::from_str(name)?;
        let key = match keys.next().map(ChannelKey::new_coerce) {
            Some(Ok(key)) => Some(key),
            Some(Err(_)) => return numeric_error!(InvalidKey, &chname),
            None => None,
        };

        let (channel_id, permissions) = match net.channel_by_name(&chname) {
            Ok(channel) => {
                server.policy().can_join(source.as_ref(), &channel, key)?;

                (channel.id(), MembershipFlagSet::new())
            }
            Err(_) => {
                let details = event::NewChannel {
                    name: chname,
                    mode: state::ChannelMode::new(ChannelModeSet::default()),
                };

                let channel_id = server.ids().next();
                cmd.new_event_with_response(channel_id, details).await;
                (channel_id, MembershipFlagFlag::Op.into())
            }
        };

        let details = event::ChannelJoin {
            user: source.id(),
            channel: channel_id,
            permissions,
        };

        let membership_id = MembershipId::new(source.id(), channel_id);
        cmd.new_event_with_response(membership_id, details).await;
    }
    Ok(())
}
