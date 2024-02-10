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
        CommandSource::User(user, user_connection) => {
            server
                .node()
                .submit_event(user_connection.id(), event::details::UserDisconnect {});
            server.add_action(CommandAction::CloseConnection(cmd.connection_id()));

            // Only quit the actual user if they're not in persistent mode
            if user.session_key().is_none() {
                server.node().submit_event(
                    user.id(),
                    event::UserQuit {
                        message: msg.unwrap_or("Client Quit").to_owned(),
                    },
                );
            }
        }
    }
    Ok(())
}
