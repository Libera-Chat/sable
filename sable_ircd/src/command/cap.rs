use super::*;
use crate::capability::*;

command_handler!("CAP" => CapHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_preclient(&mut self, source: &RefCell<PreClient>, cmd: &ClientCommand) -> CommandResult
    {
        let mut pre_client = source.borrow_mut();

        match cmd.args[0].to_ascii_uppercase().as_str()
        {
            "LS" =>
            {
                pre_client.cap_in_progress = true;

                cmd.connection.send(&message::Cap::new(self.server,
                                                       &UnknownTarget,
                                                       "LS",
                                                       self.server.client_capabilities().supported_caps()));

                Ok(())
            }
            "REQ" =>
            {
                pre_client.cap_in_progress = true;

                let requested_arg = cmd.args.get(1).ok_or_else(|| make_numeric!(NotEnoughParameters, "CAP"))?;

                if let Some(requested_caps) = self.translate_caps(requested_arg.split_whitespace())
                {
                    let mut new_caps = ClientCapabilitySet::new();
                    for cap in requested_caps
                    {
                        new_caps.set(cap);
                    }
                    self.action(CommandAction::UpdateConnectionCaps(cmd.connection.id(), new_caps))?;

                    cmd.connection.send(&message::Cap::new(self.server,
                        &UnknownTarget,
                        "ACK",
                        requested_arg));
                }
                else
                {
                    cmd.connection.send(&message::Cap::new(self.server,
                        &UnknownTarget,
                        "NAK",
                        requested_arg));
                }


                Ok(())
            }
            "END" =>
            {
                pre_client.cap_in_progress = false;
                if pre_client.can_register()
                {
                    self.action(CommandAction::RegisterClient(cmd.connection.id()))?;
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

impl<'a> CapHandler<'a>
{
    /// If all strings in `iter` are the names of supported capabilities, then return a `Vec`
    /// of the corresponding capability flags.
    ///
    /// If any string is not a supported capability name, return `None`.
    fn translate_caps<'b>(&self, iter: impl Iterator<Item=&'b str>) -> Option<Vec<ClientCapability>>
    {
        let mut ret = Vec::new();

        for item in iter
        {
            ret.push(self.server.client_capabilities().find(item)?);
        }

        Some(ret)
    }
}