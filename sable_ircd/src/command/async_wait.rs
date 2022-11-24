use std::time::Duration;

use super::*;

command_handler!("WAIT" => WaitHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user_async<'a>(&mut self, _source: UserId, cmd: Arc<ClientCommand>) -> Option<server::AsyncHandler<'a>>
    {
        Some(Box::pin(async move {
            tokio::time::sleep(Duration::from_secs(5)).await;

            cmd.connection.send(&message::Notice::new(&cmd.server, &cmd.source(), "Hello there"));

            Ok(())
        }))
    }
});