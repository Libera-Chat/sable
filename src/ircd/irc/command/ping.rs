use super::*;

command_handler!("PING", PingHandler);

impl CommandHandler for PingHandler
{
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&self, server: &Server, _source: &wrapper::User, cmd: &ClientCommand, _proc: &mut CommandProcessor) -> CommandResult
    {
        cmd.connection.send(&message::Pong::new(server, &cmd.args[0]))?;
        Ok(())
    }
}