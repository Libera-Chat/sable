use super::*;
use crate::utils::is_channel_name;
use crate::utils::make_numeric;

command_handler!("WHO" => WhoHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        if is_channel_name(&cmd.args[0])
        {
            let chname = ChannelName::new(cmd.args[0].clone())?;
            let channel = self.server.network().channel_by_name(&chname)?;
            
            self.do_who_channel(source, channel, cmd)
        }
        else
        {
            Ok(())
        }
    }
});

impl WhoHandler<'_>
{
    fn make_who_reply(&self, target: &wrapper::User, channel: Option<&wrapper::Channel>,
                      membership: Option<&wrapper::Membership>, server: &wrapper::Server) -> numeric::WhoReply
    {
        let chname = channel.map(|c| c.name()).unwrap_or("*");
        let status = format!("H{}", membership.map(|m| m.permissions().to_prefixes()).unwrap_or("".to_string()));
        make_numeric!(WhoReply, chname, target.user(), target.visible_host(), server, target.nick(), &status, 0, target.realname())
    }

    fn do_who_channel(&mut self, source: &wrapper::User, chan: wrapper::Channel, cmd: &ClientCommand) -> CommandResult
    {
        for member in chan.members()
        {
            if self.server.policy().can_see_user_on_channel(source, &member).is_err()
            {
                continue;
            }

            let reply = self.make_who_reply(&member.user()?, Some(&chan), Some(&member), &self.server.me()?);
            cmd.connection.send(&reply.format_for(self.server, source));
        }

        let endofwho = make_numeric!(EndOfWho, chan.name());
        cmd.connection.send(&endofwho.format_for(self.server, source));

        Ok(())
    }
}