use super::*;

#[command_handler("NS")]
async fn handle_cs<'a>(_source: UserSource<'a>, subcommand: &'a str, args: ArgList<'a>) -> CommandResult
{
    let new_context = ServicesCommand::new(args.context(), subcommand, args.iter());

    let dispatcher = CommandDispatcher::with_category("NS");

    if let Some(future) = dispatcher.dispatch_command(new_context)
    {
        future.await;
    }

    Ok(())
}

mod login;