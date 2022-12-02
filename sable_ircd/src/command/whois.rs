use super::*;

command_handler!("WHOIS" => WhoisHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let target_nick = Nickname::from_str(&cmd.args[0])?;
        let net = self.server.network();
        let target = net.user_by_nick(&target_nick)?;

        cmd.connection.send(&make_numeric!(WhoisUser, &target).format_for(&self.server, source));
        cmd.connection.send(&make_numeric!(WhoisServer, &target, &target.server()?)
                                .format_for(&self.server, source));

        if let Ok(Some(account)) = target.account()
        {
            cmd.connection.send(&make_numeric!(WhoisAccount, &target, &account.name()).format_for(&self.server, source));
        }

        cmd.connection.send(&make_numeric!(EndOfWhois, &target)
                                .format_for(&self.server, source));
        Ok(())
    }
});