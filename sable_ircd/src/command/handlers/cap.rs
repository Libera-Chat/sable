use super::*;
use crate::capability::*;

use std::sync::atomic::Ordering;

#[command_handler("CAP")]
fn handle_cap(server: &ClientServer, pre_client: PreClientSource, cmd: &dyn Command,
              subcommand: &str, cap_list: Option<&str>) -> CommandResult
{
    match subcommand.to_ascii_uppercase().as_str()
    {
        "LS" =>
        {
            pre_client.cap_in_progress.store(true, Ordering::Relaxed);

            cmd.response(&message::Cap::new(&server,
                                            &UnknownTarget,
                                            "LS",
                                            server.client_capabilities().supported_caps()));

            Ok(())
        }
        "REQ" =>
        {
            pre_client.cap_in_progress.store(true, Ordering::Relaxed);

            let requested_arg = cap_list.ok_or_else(|| make_numeric!(NotEnoughParameters, "CAP"))?;

            if let Some(requested_caps) = translate_caps(server, requested_arg.split_whitespace())
            {
                let mut new_caps = ClientCapabilitySet::new();
                for cap in requested_caps
                {
                    new_caps.set(cap);
                }
                server.add_action(CommandAction::UpdateConnectionCaps(cmd.connection(), new_caps));

                cmd.response(&message::Cap::new(&server,
                                                &UnknownTarget,
                                                "ACK",
                                                requested_arg));
            }
            else
            {
                cmd.response(&message::Cap::new(server,
                                                &UnknownTarget,
                                                "NAK",
                                                requested_arg));
            }


            Ok(())
        }
        "END" =>
        {
            pre_client.cap_in_progress.store(false, Ordering::Relaxed);
            if pre_client.can_register()
            {
                server.add_action(CommandAction::RegisterClient(cmd.connection()));
            }
            Ok(())
        }
        _ =>
        {
            Ok(())
        }
    }
}

/// If all strings in `iter` are the names of supported capabilities, then return a `Vec`
/// of the corresponding capability flags.
///
/// If any string is not a supported capability name, return `None`.
fn translate_caps<'b>(server: &ClientServer, iter: impl Iterator<Item=&'b str>) -> Option<Vec<ClientCapability>>
{
    let mut ret = Vec::new();

    for item in iter
    {
        ret.push(server.client_capabilities().find(item)?);
    }

    Some(ret)
}
