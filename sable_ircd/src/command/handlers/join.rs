use super::*;

#[command_handler("JOIN")]
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

                let channel_id = server.ids().next_channel();
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
