use super::*;

/// Minimal implementation of TAGMSG that just drops everything, so we can safely
/// implement <https://ircv3.net/specs/extensions/message-tags>
///
/// `CLIENTTAGDENY=*` tells clients we would drop all tags, anyway.
#[command_handler("TAGMSG")]
fn handle_tagmsg() -> CommandResult {
    Ok(())
}
