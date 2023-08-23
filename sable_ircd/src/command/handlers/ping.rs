use super::*;

#[command_handler("PING")]
fn handle_ping(server: &ClientServer, response: CommandResponse, cookie: &str) -> CommandResult
{
    response.send(message::Pong::new(server, cookie));
    Ok(())
}
