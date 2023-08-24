use super::*;
use std::time::Duration;

#[command_handler("WAIT")]
async fn handle_wait(cmd: &ClientCommand, _source: UserSource) -> CommandResult {
    tokio::time::sleep(Duration::from_secs(5)).await;

    cmd.connection.send(&message::Notice::new(
        &cmd.server,
        &cmd.source(),
        "Hello there",
    ));

    Ok(())
}
