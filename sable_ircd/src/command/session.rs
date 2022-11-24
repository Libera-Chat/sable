use super::*;
//use crate::capability::*;

command_handler!("SESSION" => SessionHandler {
    fn min_parameters(&self) -> usize
    {
        1
    }
/*
    fn required_capabilities(&self) -> ClientCapabilitySet
    {
        ClientCapability::PersistentSession.into()
    }
*/

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let subcommand = cmd.args[0].to_ascii_uppercase();

        match subcommand.as_str()
        {
            "ENABLE" =>
            {
                let key_input = format!("{:?}{}", source.id(), rand::random::<u64>());
                let key_hash = sha256::digest(key_input);

                cmd.connection.send(&message::Notice::new(&self.server, source,
                    &format!("Your session resumption token is {}", key_hash)));

                self.action(CommandAction::StateChange(source.id().into(),
                    event::details::EnablePersistentSession {
                        key_hash
                    }.into()
                ))?;

                Ok(())
            }
            _ =>
            {
                Ok(())
            }
        }
    }

    fn handle_preclient(&mut self, _source: &PreClient, cmd: &ClientCommand) -> CommandResult
    {
        let subcommand = cmd.args[0].to_ascii_uppercase();

        match subcommand.as_str()
        {
            "ATTACH" =>
            {
                let key = &cmd.args[1];

                if let Some(target_user) = self.server.network().raw_users()
                                                .find(|u| matches!(&u.session_key, Some(sk) if &sk.key_hash == key))
                {
                    self.action(CommandAction::AttachToUser(cmd.connection.id(), target_user.id))?;
                }

                Ok(())
            }
            _ =>
            {
                Ok(())
            }
        }
    }
});