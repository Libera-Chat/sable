use super::*;

#[command_handler("CS")]
async fn handle_cs(source: UserSource<'_>, cmd: &ClientCommand) -> CommandResult
{
    let subcommand = cmd.args[0].to_ascii_uppercase();

    match subcommand.as_str()
    {
        "REGISTER" => register::handle_register(&source, cmd).await,
        "ACCESS" => access::handle_access(&source, cmd).await,
        "ROLE" => role::handle_role(&source, cmd).await,
        _ =>
        {
            cmd.notice(format_args!("Unrecognised CS command {}", subcommand));
            Ok(())
        }
    }
}

mod register;
mod access;
mod role;