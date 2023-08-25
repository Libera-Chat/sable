use super::*;

#[command_handler("JOIN")]
fn handle_join(
    server: &ClientServer,
    net: &Network,
    source: UserSource,
    channel_names: &str,
    keys: Option<&str>,
) -> CommandResult {
    let empty_str = String::new();
    let names = channel_names.split(',');
    let mut keys = keys.unwrap_or(&empty_str).split(',');

    for name in names {
        let chname = ChannelName::from_str(name)?;
        let key = keys.next().map(ChannelKey::new_coerce);

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
                server.add_action(CommandAction::state_change(channel_id, details));
                (channel_id, MembershipFlagFlag::Op.into())
            }
        };

        let details = event::ChannelJoin {
            user: source.id(),
            channel: channel_id,
            permissions,
        };

        let membership_id = MembershipId::new(source.id(), channel_id);
        server.add_action(CommandAction::state_change(membership_id, details));
    }
    Ok(())
}
