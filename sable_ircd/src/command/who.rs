use super::*;
use sable_network::utils::is_channel_name;
use crate::utils::make_numeric;

command_handler!("WHO" => WhoHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        if is_channel_name(&cmd.args[0])
        {
            let chname = ChannelName::from_str(&cmd.args[0])?;
            let net = self.server.network();
            let channel = net.channel_by_name(&chname)?;

            self.do_who_channel(source, channel, cmd)
        }
        else
        {
            Ok(())
        }
    }
});

impl WhoHandler
{
    fn make_who_reply(&self, target: &wrapper::User, channel: Option<&wrapper::Channel>,
                      membership: Option<&wrapper::Membership>, server: &wrapper::Server) -> numeric::WhoReply
    {
        let chname = channel.map(|c| c.name().value() as &str).unwrap_or("*");
        let status = format!("H{}", membership.map(|m| m.permissions().to_prefixes()).unwrap_or_else(|| "".to_string()));
        make_numeric!(WhoReply, chname, target, server, &status, 0)
    }

    fn do_who_channel(&mut self, source: &wrapper::User, chan: wrapper::Channel, cmd: &ClientCommand) -> CommandResult
    {
        for member in chan.members()
        {
            if self.server.policy().can_see_user_on_channel(source, &member).is_err()
            {
                continue;
            }

            let reply = self.make_who_reply(&member.user()?, Some(&chan), Some(&member), &member.user()?.server()?);
            cmd.connection.send(&reply.format_for(&self.server, source));
        }

        let endofwho = make_numeric!(EndOfWho, chan.name().value());
        cmd.connection.send(&endofwho.format_for(&self.server, source));

        Ok(())
    }
}