use super::*;

command_handler!("NAMES" => NamesHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let chname = ChannelName::from_str(&cmd.args[0])?;
        let net = self.server.network();
        let channel = net.channel_by_name(&chname)?;

        crate::utils::send_channel_names(self.server, cmd.connection, &source, &channel)?;

        Ok(())
    }
});