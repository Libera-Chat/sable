use super::*;
use event::*;
use state::{
    AuditLogCategory,
    AuditLogField
};

#[command_handler("KILL")]
fn handle_kill(server: &ClientServer, source: UserSource,
               target: wrapper::User, message: &str) -> CommandResult
{
    server.policy().require_oper(&source)?;

    server.policy().can_kill(&source, &target)?;

    let audit = details::NewAuditLogEntry {
        category: AuditLogCategory::General,
        fields: vec![
            (AuditLogField::Source, source.nuh()),
            (AuditLogField::ActionType, "KILL".to_string()),
            (AuditLogField::TargetUser, target.nuh()),
            (AuditLogField::Reason, message.to_string())
        ]
    };
    server.add_action(CommandAction::state_change(server.ids().next_audit_log_entry(), audit));

    server.add_action(CommandAction::state_change(target.id(), details::UserQuit {
        message: format!("Killed by {} ({})", source.nick(), message)
    }));

    Ok(())
}

