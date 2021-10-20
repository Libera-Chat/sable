use super::*;
use crate::utils::is_channel_name;

command_handler!("PRIVMSG", PrivmsgHandler);

impl CommandHandler for PrivmsgHandler
{
    fn min_parameters(&self) -> usize { 2 }

    fn handle_user(&self, server: &Server, source: &wrapper::User, cmd: &ClientCommand, proc: &mut CommandProcessor) -> CommandResult
    {
        let target_name = &cmd.args[0];
        let target_id = if is_channel_name(target_name) {
            ObjectId::Channel(server.network().channel_by_name(target_name)?.id())
        } else {
            ObjectId::User(server.network().user_by_nick(target_name)?.id())
        };

        let details = event::details::NewMessage {
            source: source.id(),
            target: target_id,
            text: cmd.args[1].clone(),
        };
        proc.action(CommandAction::StateChange(server.create_event(server.next_message_id(), details))).translate(cmd)?;
        Ok(())
    }
}