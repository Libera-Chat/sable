use super::*;

command_handler!("NAMES" => NamesHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&mut self, _source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let chname = ChannelName::new(cmd.args[0].clone())?;
        let channel = self.server.network().channel_by_name(&chname)?;

        irc::utils::send_channel_names(self.server, cmd.connection, &channel)?;

        Ok(())
    }
});