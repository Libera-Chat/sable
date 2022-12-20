use super::*;

#[command_handler("NS")]
async fn handle_ns(source: UserSource<'_>, cmd: &ClientCommand) -> CommandResult
{
    let subcommand = cmd.args[0].to_ascii_uppercase();

    match subcommand.as_str()
    {
        "ID"|"IDENTIFY"|"LOGIN" => login::handle_login(&source, cmd).await,
        _ =>
        {
            cmd.notice(format_args!("Unrecognised NS command {}", subcommand));
            Ok(())
        }
    }
}

mod login;