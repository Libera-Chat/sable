use super::*;
use event::*;
use state::{
    AuditLogCategory,
    AuditLogField
};

command_handler!("KILL" => KillHandler {
    fn min_parameters(&self) -> usize { 2 }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        self.server.policy().require_oper(source)?;

        let target_nick = &cmd.args[0];
        let message = &cmd.args[1];

        let net = self.server.network();
        let target = net.user_by_nick(&Nickname::from_str(target_nick)?)?;
        self.server.policy().can_kill(source, &target)?;

        let audit = details::NewAuditLogEntry {
            category: AuditLogCategory::General,
            fields: vec![
                (AuditLogField::Source, source.nuh()),
                (AuditLogField::ActionType, "KILL".to_string()),
                (AuditLogField::TargetUser, target.nuh()),
                (AuditLogField::Reason, message.clone())
            ]
        };
        self.action(CommandAction::state_change(self.server.ids().next_audit_log_entry(), audit))?;

        self.action(CommandAction::state_change(target.id(), details::UserQuit {
            message: format!("Killed by {} ({})", source.nick(), message)
        }))?;

        Ok(())
    }
});