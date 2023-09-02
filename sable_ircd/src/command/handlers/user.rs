use super::*;

#[command_handler("USER")]
fn handle_user(
    server: &ClientServer,
    source: PreClientSource,
    cmd: &dyn Command,
    username: Username,
    _unused1: &str,
    _unused2: &str,
    realname: Result<Realname, &str>,
) -> CommandResult {
    let Ok(realname) = realname else {
        /* "If it is empty, the server SHOULD reject the command with ERR_NEEDMOREPARAMS
         * (even if an empty parameter is provided);"
         * -- https://modern.ircdocs.horse/#user-message
         */
        return numeric_error!(NotEnoughParameters, "USER");
    };

    // Ignore these results; they'll only fail if USER was already successfully processed
    // from this pre-client. If that happens we silently ignore the new values.
    source.user.set(username).ok();
    source.realname.set(realname).ok();

    if source.can_register() {
        server.add_action(CommandAction::RegisterClient(cmd.connection_id()));
    }
    Ok(())
}
