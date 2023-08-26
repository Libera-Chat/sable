use super::*;
use crate::capability::*;

#[command_handler("CAP")]
fn handle_cap(
    server: &ClientServer,
    pre_client: PreClientSource,
    cmd: &dyn Command,
    response: &dyn CommandResponse,
    subcommand: &str,
    cap_list: Option<&str>,
) -> CommandResult {
    match subcommand.to_ascii_uppercase().as_str() {
        "LS" => {
            pre_client.start_progress(ProgressFlag::CapNegotiation);

            if matches!(cap_list, Some("302")) {
                response.send(message::Cap::new(
                    &server,
                    &UnknownTarget,
                    "LS",
                    server.client_capabilities().supported_caps_302().as_ref(),
                ));
            } else {
                response.send(message::Cap::new(
                    &server,
                    &UnknownTarget,
                    "LS",
                    server.client_capabilities().supported_caps_301().as_ref(),
                ));
            }

            Ok(())
        }
        "REQ" => {
            pre_client.start_progress(ProgressFlag::CapNegotiation);

            let requested_arg =
                cap_list.ok_or_else(|| make_numeric!(NotEnoughParameters, "CAP"))?;

            if let Some(requested_caps) = translate_caps(server, requested_arg.split_whitespace()) {
                let mut new_caps = ClientCapabilitySet::new();
                for cap in requested_caps {
                    new_caps.set(cap);
                }
                server.add_action(CommandAction::UpdateConnectionCaps(
                    cmd.connection_id(),
                    new_caps,
                ));

                response.send(message::Cap::new(
                    &server,
                    &UnknownTarget,
                    "ACK",
                    requested_arg,
                ));
            } else {
                response.send(message::Cap::new(
                    server,
                    &UnknownTarget,
                    "NAK",
                    requested_arg,
                ));
            }

            Ok(())
        }
        "END" => {
            if pre_client.complete_progress(ProgressFlag::CapNegotiation) {
                server.add_action(CommandAction::RegisterClient(cmd.connection_id()));
            }
            Ok(())
        }
        _ => {
            response.numeric(make_numeric!(InvalidCapCmd, subcommand));
            Ok(())
        }
    }
}

/// If all strings in `iter` are the names of supported capabilities, then return a `Vec`
/// of the corresponding capability flags.
///
/// If any string is not a supported capability name, return `None`.
fn translate_caps<'b>(
    server: &ClientServer,
    iter: impl Iterator<Item = &'b str>,
) -> Option<Vec<ClientCapability>> {
    let mut ret = Vec::new();

    for item in iter {
        ret.push(server.client_capabilities().find(item)?);
    }

    Some(ret)
}
