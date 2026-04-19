use super::*;
use event::*;

#[command_handler("AWAY")]
/// AWAY :[<reason>]
///
/// With an argument, it will set you as AWAY with
/// the specified message. Without an argument,
/// it will set you back.
async fn away_handler(
    cmd: &dyn Command,
    source: UserSource<'_>,
    reason: Option<AwayReason>,
) -> CommandResult {
    let detail = details::UserAway { reason };

    cmd.new_event_with_response(source.id(), detail).await;
    Ok(())
}
