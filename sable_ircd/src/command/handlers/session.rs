use super::*;
//use crate::capability::*;

#[command_handler("SESSION")]
fn handle_session(server: &ClientServer, cmd: &ClientCommand, source: CommandSource,
                  subcommand: &str, key: Conditional<&str>) -> CommandResult
{
    match (source, subcommand.to_ascii_uppercase().as_str())
    {
        (CommandSource::User(user), "ENABLE") =>
        {
            let key_input = format!("{:?}{}", user.id(), rand::random::<u64>());
            let key_hash = sha256::digest(key_input);

            cmd.notice(format_args!("Your session resumption token is {}", key_hash));

            server.add_action(CommandAction::state_change(user.id(),
                event::details::EnablePersistentSession {
                    key_hash
                }
            ));

            Ok(())
        }
        (CommandSource::PreClient(_), "ATTACH") =>
        {
            let key = key.require()?;

            if let Some(target_user) = server.network().raw_users()
                                            .find(|u| matches!(&u.session_key, Some(sk) if &sk.key_hash == key))
            {
                server.add_action(CommandAction::AttachToUser(cmd.connection.id(), target_user.id));
            }

            Ok(())
        }
        _ =>
        {
            Ok(())
        }
    }
}
