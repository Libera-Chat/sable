use super::*;

#[command_handler("TAGMSG")]
async fn handle_tagmsg(_target: TargetParameter<'_>) -> CommandResult {
    // Ignore, no client tag is supported yet.
    //
    // We send CLIENTTAGDENY=* in ISUPPORT, so well-behaved clients should never send
    // TAGMSGs anyway.
    Ok(())
}
