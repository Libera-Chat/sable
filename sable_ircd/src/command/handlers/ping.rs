use super::*;

#[command_handler("PING")]
fn handle_ping(server: &ClientServer, cmd: &dyn Command, cookie: &str) -> CommandResult
{
    cmd.response(message::Pong::new(server, cookie));
    Ok(())
}
