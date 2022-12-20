use super::*;

#[command_handler("NAMES")]
fn handle_names(server: &ClientServer, cmd: &ClientCommand, source: UserSource,
               channel: wrapper::Channel) -> CommandResult
{
    crate::utils::send_channel_names(server, &*cmd.connection, &source, &channel)?;

    Ok(())
}
