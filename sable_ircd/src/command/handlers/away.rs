use super::*;
use event::*;

#[command_handler("AWAY")]
async fn away_handler(
    cmd: &dyn Command,
    source: UserSource<'_>,
    reason: Option<&str>,
) -> CommandResult {
    // Empty reason means not away
    let reason = reason.unwrap_or("").to_string();

    let detail = details::UserAway { reason };

    cmd.new_event_with_response(source.id(), detail).await;
    Ok(())
}
