use super::*;

command_handler!("WHOIS" => WhoisHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let target_nick = Nickname::new(cmd.args[0].clone())?;
        let target = self.server.network().user_by_nick(&target_nick)?;

        cmd.connection.send(&make_numeric!(WhoisUser, &target, &target, &target, &target)
                                .format_for(self.server, source));
        cmd.connection.send(&make_numeric!(WhoisServer, &target, &target.server()?, &target.server()?)
                                .format_for(self.server, source));
        cmd.connection.send(&make_numeric!(EndOfWhois, &target)
                                .format_for(self.server, source));
        Ok(())
    }
});