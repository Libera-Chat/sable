use super::*;

command_handler!("PING" => PingHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&mut self, _source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        cmd.connection.send(&message::Pong::new(self.server, &cmd.args[0]));
        Ok(())
    }
});