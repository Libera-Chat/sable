use super::*;

#[command_handler("CS")]
async fn handle_cs<'a>(_source: UserSource<'a>, subcommand: &'a str, args: ArgList<'a>) -> CommandResult
{
    let new_context = ServicesCommand::new(args.context(), subcommand, args.iter(), None);

    let dispatcher = CommandDispatcher::with_category("CS");

    if let Some(future) = dispatcher.dispatch_command(new_context)
    {
        future.await;
    }

    Ok(())
}

mod register;
mod access;
mod role;