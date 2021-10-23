use super::*;

command_handler!("MODE", ModeHandler);

impl CommandHandler for ModeHandler
{
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&self, server: &Server, _source: &wrapper::User, cmd: &ClientCommand, _proc: &mut CommandProcessor) -> CommandResult
    {
        let target = cmd.args[0].clone();
        if let Ok(cname) = ChannelName::new(target)
        {
            let chan = server.network().channel_by_name(&cname)?;
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
}