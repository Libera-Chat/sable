use super::*;

#[allow(clippy::too_many_arguments)]
#[command_handler("RENAME")]
async fn handle_rename(
    server: &ClientServer,
    net: &Network,
    cmd: &dyn Command,
    response: &dyn CommandResponse,
    source: UserSource<'_>,
    channel: wrapper::Channel<'_>,
    new_name: &str,
    message: Option<&str>,
) -> CommandResult {
    let new_name = ChannelName::from_str(new_name)?;
    server
        .policy()
        .can_rename(source.as_ref(), &channel, &new_name, message)?;

    if net.channel_by_name(&new_name).is_ok() {
        response.send(message::Fail::new(
            "RENAME",
            "CHANNEL_NAME_IN_USE",
            &format!("{} {}", channel.name(), new_name), // two context params
            "The channel name is already taken",
        ));
        return Ok(());
    }

    let details = event::ChannelRename {
        source: source.id(),
        new_name,
        message: message.map(|s| s.to_owned()),
    };

    cmd.new_event_with_response(channel.id(), details).await;
    Ok(())
}
