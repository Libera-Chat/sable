use super::*;

#[command_handler("USER")]
fn handle_user(
    server: &ClientServer,
    source: PreClientSource,
    cmd: &dyn Command,
    username: &str,
    _unused1: &str,
    _unused2: &str,
    realname: &str,
) -> CommandResult {
    if realname.len() == 0 {
        /* "The minimum length of <username> is 1, ie. it MUST NOT be empty. If it is
         * empty, the server SHOULD reject the command with ERR_NEEDMOREPARAMS (even if
         * an empty parameter is provided)"
         * -- https://modern.ircdocs.horse/#user-message
         */
        return numeric_error!(NotEnoughParameters, "USER");
    }
    // Ignore these results; they'll only fail if USER was already successfully processed
    // from this pre-client. If that happens we silently ignore the new values.
    source.user.set(Username::new_coerce(username)).ok();
    source.realname.set(realname.to_owned()).ok();

    if source.can_register() {
        server.add_action(CommandAction::RegisterClient(cmd.connection_id()));
    }
    Ok(())
}
