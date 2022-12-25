use super::*;

#[command_handler("NAMES")]
fn handle_names(server: &ClientServer, cmd: &dyn Command, source: UserSource,
               channel: wrapper::Channel) -> CommandResult
{
    crate::utils::send_channel_names(server, cmd, &source, &channel)?;

    Ok(())
}
