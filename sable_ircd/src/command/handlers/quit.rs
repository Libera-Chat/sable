use super::*;

#[command_handler("QUIT")]
fn handle_quit(
    server: &ClientServer,
    source: UserSource,
    response: &dyn CommandResponse,
    msg: Option<&str>,
) -> CommandResult {
    response.send(message::Error::new("Client quit"));
    server.add_action(CommandAction::DisconnectUser(source.id()));
    server.add_action(CommandAction::state_change(
        source.id(),
        event::UserQuit {
            message: msg.unwrap_or("Client Quit").to_owned(),
        },
    ));
    Ok(())
}
