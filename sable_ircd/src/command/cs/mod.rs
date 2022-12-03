use super::*;

command_handler!("CS" => CSHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user_async<'a>(&mut self, source: UserId, cmd: Arc<ClientCommand>) -> Option<server::AsyncHandler<'a>>
    {
        Some(Box::pin(async move {
            let subcommand = cmd.args[0].to_ascii_uppercase();

            match subcommand.as_str()
            {
                "REGISTER" => register::handle_register(source, cmd).await,
                _ =>
                {
                    cmd.notice(format_args!("Unrecognised CS command {}", subcommand));
                    Ok(())
                }
            }
        }))
    }
});

mod register;
