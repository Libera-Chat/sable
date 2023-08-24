use super::*;

#[command_handler("PING")]
fn handle_ping(
    server: &ClientServer,
    response: &dyn CommandResponse,
    cookie: &str,
) -> CommandResult {
    response.send(message::Pong::new(server, cookie));
    Ok(())
}
