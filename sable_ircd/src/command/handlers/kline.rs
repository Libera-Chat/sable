use super::*;
use event::*;
use state::{
    AuditLogCategory,
    AuditLogField
};

const DEFAULT_KLINE_DURATION: u32 = 1440;

#[command_handler("KLINE")]
fn handle_kline(server: &ClientServer, cmd: &dyn Command, source: UserSource,
                duration: IfParses<u32>, mask: &str, message: &str) -> CommandResult
{
    server.policy().require_oper(&source)?;

    let duration = duration.unwrap_or(DEFAULT_KLINE_DURATION);

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
                (AuditLogField::NetworkBanMask, mask.to_string()),
                (AuditLogField::NetworkBanDuration, duration.to_string()),
                (AuditLogField::Reason, message.to_string()),
            ]
        };
        server.add_action(CommandAction::state_change(server.ids().next_audit_log_entry(), audit));

        let new_kline = details::NewKLine {
            user: Pattern::new(user.to_string()),
            host: Pattern::new(host.to_string()),
            setter: source.id(),
            duration: (duration * 60) as i64,
            user_reason: user_reason.to_string(),
            oper_reason: oper_reason.map(|s| s.to_string())
        };
        server.add_action(CommandAction::state_change(server.ids().next_network_ban(), new_kline));
    }
    else
    {
        cmd.notice("Invalid kline mask");
    }

    Ok(())
}

