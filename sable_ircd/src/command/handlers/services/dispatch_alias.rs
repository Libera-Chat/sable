use super::*;

pub async fn dispatch_alias_command(
    cmd: &dyn Command,
    through_user: &wrapper::User<'_>,
    alias: &str,
    command_str: &str,
) -> CommandResult {
    let (command, args) = command_str.split_once(" ").unwrap_or((command_str, ""));

    let new_args = args.split(" ").map(ToOwned::to_owned).collect::<Vec<_>>();
    let new_arg_iter = ArgListIter::new(&new_args);

    let new_cmd = ServicesCommand::new(cmd, command, new_arg_iter, Some(through_user));
    let dispatcher = CommandDispatcher::with_category(alias);

    if let Some(future) = dispatcher.dispatch_command(new_cmd) {
        future.await;
    }

    Ok(())
}
