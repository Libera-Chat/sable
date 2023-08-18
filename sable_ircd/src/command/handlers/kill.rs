use super::*;
use event::*;

#[command_handler("KILL")]
fn handle_kill(server: &ClientServer, source: UserSource, audit: AuditLogger,
               target: wrapper::User, message: &str) -> CommandResult
{
    server.policy().require_oper(&source)?;

    server.policy().can_kill(&source, &target)?;

    audit.general().target_user(target.id()).reason(message.to_string()).log();

    server.add_action(CommandAction::state_change(target.id(), details::UserQuit {
        message: format!("Killed by {} ({})", source.nick(), message)
    }));

    Ok(())
}

