use super::*;
//use crate::capability::*;

#[command_handler("SESSION")]
fn handle_session(
    server: &ClientServer,
    cmd: &dyn Command,
    response: &dyn CommandResponse,
    source: CommandSource,
    subcommand: &str,
    key: Conditional<&str>,
) -> CommandResult {
    match (source, subcommand.to_ascii_uppercase().as_str()) {
        (CommandSource::User(user, _), "ENABLE") => {
            let key_input = format!("{:?}{}", user.id(), rand::random::<u64>());
            let key_hash = sha256::digest(key_input);

            response.notice(&format!("Your session resumption token is {}", key_hash));

            server.add_action(CommandAction::state_change(
                user.id(),
                event::details::EnablePersistentSession { key_hash },
            ));

            Ok(())
        }
        (CommandSource::PreClient(pre_client), "ATTACH") => {
            let key = key.require()?;

            if let Some(target_user) = server
                .network()
                .raw_users()
                .find(|u| matches!(&u.session_key, Some(sk) if &sk.key_hash == key))
            {
                // Ok to ignore an error here, as that'll only happen if the command is run twice
                let _ = pre_client.attach_user_id.set(target_user.id);

                if pre_client.can_register() {
                    server.add_action(CommandAction::RegisterClient(cmd.connection_id()));
                }
            }

            Ok(())
        }
        _ => Ok(()),
    }
}
