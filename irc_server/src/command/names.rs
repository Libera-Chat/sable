use super::*;

command_handler!("NAMES" => NamesHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&mut self, _source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let chname = ChannelName::from_str(&cmd.args[0])?;
        let channel = self.server.network().channel_by_name(&chname)?;

        crate::utils::send_channel_names(self.server, cmd.connection, &channel)?;

        Ok(())
    }
});