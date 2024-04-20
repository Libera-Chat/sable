use super::*;

#[command_handler("NAMES")]
fn handle_names(
    server: &ClientServer,
    response: &dyn CommandResponse,
    source: UserSource,
    channel: Result<wrapper::Channel, &str>,
) -> CommandResult {
    match channel {
        Ok(channel) => Ok(crate::utils::send_channel_names(
            server, response, &source, &channel,
        )?),
        Err(channel_name) => {
            // "If the channel name is invalid or the channel does not exist, one RPL_ENDOFNAMES numeric
            // containing the given channel name should be returned." -- https://modern.ircdocs.horse/#names-message
            response.numeric(make_numeric!(EndOfNames, channel_name));
            Ok(())
        }
    }
}
