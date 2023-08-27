use super::*;

#[command_handler("QUIT")]
fn handle_quit(
    server: &ClientServer,
    cmd: &dyn Command,
    source: CommandSource,
    response: &dyn CommandResponse,
    msg: Option<&str>,
) -> CommandResult {
    response.send(message::Error::new("Client quit"));
    match source {
        CommandSource::PreClient(_) => {
            server.add_action(CommandAction::CloseConnection(cmd.connection_id()));
        }
        CommandSource::User(user) => {
            server.add_action(CommandAction::DisconnectUser(user.id()));
            server.add_action(CommandAction::state_change(
                user.id(),
                event::UserQuit {
                    message: msg.unwrap_or("Client Quit").to_owned(),
                },
            ));
        }
    }
    Ok(())
}
