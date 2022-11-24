use super::*;
use event::*;
use state::{
    AuditLogCategory,
    AuditLogField
};

command_handler!("KLINE" => KlineHandler {
    fn min_parameters(&self) -> usize { 3 }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        self.server.policy().require_oper(source)?;

        let duration = if let Ok(i) = cmd.args[0].parse::<i64>() {
            i
        } else {
            cmd.connection.send(&message::Notice::new(&self.server, source, "Invalid duration"));
            return Ok(());
        };

        let mask = &cmd.args[1];
        let message = &cmd.args[2];

        let parts:Vec<_> = message.splitn(2, '|').collect();
        let user_reason = parts[0];
        let oper_reason = parts.get(1);

        let mask_parts:Vec<_> = mask.split('@').collect();

        if let [user, host] = mask_parts[..]
        {
            let audit = details::NewAuditLogEntry {
                category: AuditLogCategory::NetworkBan,
                fields: vec![
                    (AuditLogField::Source, source.nuh()),
                    (AuditLogField::ActionType, "KLINE".to_string()),
                    (AuditLogField::NetworkBanMask, mask.clone()),
                    (AuditLogField::NetworkBanDuration, duration.to_string()),
                    (AuditLogField::Reason, message.clone()),
                ]
            };
            self.action(CommandAction::state_change(self.server.ids().next_audit_log_entry(), audit))?;

            let new_kline = details::NewKLine {
                user: Pattern::new(user.to_string()),
                host: Pattern::new(host.to_string()),
                setter: source.id(),
                duration: duration * 60,
                user_reason: user_reason.to_string(),
                oper_reason: oper_reason.map(|s| s.to_string())
            };
            self.action(CommandAction::state_change(self.server.ids().next_network_ban(), new_kline))?;
        }
        else
        {
            cmd.connection.send(&message::Notice::new(&self.server, source, "Invalid mask"));
        }

        Ok(())
    }
});