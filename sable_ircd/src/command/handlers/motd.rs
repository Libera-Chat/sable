use super::*;

#[command_handler("MOTD")]
fn handle_motd(
    server: &ClientServer,
    response: &dyn CommandResponse,
    source: UserSource,
) -> CommandResult {
    crate::utils::send_motd(server, response, &source)?;
    Ok(())
}
