use super::*;

#[command_handler("PING")]
fn handle_ping(server: &ClientServer, cmd: &ClientCommand, cookie: &str) -> CommandResult
{
    cmd.response(&message::Pong::new(server, cookie));
    Ok(())
}
