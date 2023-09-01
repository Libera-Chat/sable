use super::*;

#[command_handler("NAMES")]
fn handle_names(
    server: &ClientServer,
    response: &dyn CommandResponse,
    source: UserSource,
    channel: &str,
) -> CommandResult {
    if let Ok(channel_name) = &ChannelName::from_str(channel) {
        if let Ok(channel) = server.network().channel_by_name(channel_name) {
            crate::utils::send_channel_names(server, &response, &source, &channel)?;
            return Ok(());
        }
    }

    // "If the channel name is invalid or the channel does not exist, one RPL_ENDOFNAMES numeric
    // containing the given channel name should be returned." -- https://modern.ircdocs.horse/#names-message
    response.numeric(make_numeric!(EndOfNames, channel));

    Ok(())
}
