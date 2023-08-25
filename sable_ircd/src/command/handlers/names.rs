use super::*;

#[command_handler("NAMES")]
fn handle_names(
    server: &ClientServer,
    response: &dyn CommandResponse,
    source: UserSource,
    channel: wrapper::Channel,
) -> CommandResult {
    crate::utils::send_channel_names(server, &response, &source, &channel)?;

    Ok(())
}
