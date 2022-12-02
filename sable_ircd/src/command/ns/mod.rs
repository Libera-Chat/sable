use super::*;

command_handler!("NS" => NSHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user_async<'a>(&mut self, source: UserId, cmd: Arc<ClientCommand>) -> Option<server::AsyncHandler<'a>>
    {
        Some(Box::pin(async move {
            let subcommand = cmd.args[0].to_ascii_uppercase();

            match subcommand.as_str()
            {
                "ID"|"IDENTIFY"|"LOGIN" => login::handle_login(source, cmd).await,
                _ =>
                {
                    let msg = format!("Unrecognised NS command {}", subcommand);
                    cmd.notice(msg);
                    Ok(())
                }
            }
        }))
    }
});

mod login;