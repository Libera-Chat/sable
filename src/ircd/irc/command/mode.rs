use super::*;

command_handler!("MODE" => ModeHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&mut self, _source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let target = cmd.args[0].clone();
        if let Ok(cname) = ChannelName::new(target)
        {
            let chan = self.server.network().channel_by_name(&cname)?;
            let mode = chan.mode()?;

            let msg = numeric::ChannelModeIs::new(&chan, &mode);
            cmd.response(&msg)?;
        }
        else
        {
            log::error!("User modes not implemented");
        }
        Ok(())
    }
});