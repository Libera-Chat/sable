use super::*;

command_handler!("SPING" => SPingHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user_async<'a>(&mut self, _source: UserId, cmd: Arc<ClientCommand>) -> Option<server::AsyncHandler>
    {
        Some(Box::pin(async move {
            let Ok(target_name) = ServerName::from_str(&cmd.args[0]) else {
                cmd.connection.send(&message::Notice::new(&cmd.server, &cmd.source(), "Invalid server name"));
                return Ok(());
            };

            match cmd.server.server().sync_log().send_remote_request(target_name, rpc::RemoteServerRequestType::Ping).await
            {
                Ok(_response) => {
                    let msg = format!("Got response from {}", target_name);
                    cmd.connection.send(&message::Notice::new(&cmd.server, &cmd.source(), &msg));
                }
                Err(e) => {
                    let msg = format!("Error contacting remote server: {}", e);
                    cmd.connection.send(&message::Notice::new(&cmd.server, &cmd.source(), &msg));
                }
            }

            Ok(())
        }))
    }
});
