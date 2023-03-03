use super::*;
use sable_network::network::ban::*;
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

        let matcher = match NetworkBanMatch::from_user_host(user, host) {
            Ok(matcher) => matcher,
            Err(_) => {
                cmd.notice(format_args!("A network ban is already set on {}", mask));
                return Ok(())
            }
        };

        let new_kline = state::NetworkBan {
            id: server.ids().next_network_ban(),
            matcher,
            action: NetworkBanAction::RefuseConnection(true),
            setter_info: source.0.nuh(),
            timestamp: sable_network::utils::now(),
            expires: sable_network::utils::now() + (duration * 60) as i64,
            reason: user_reason.to_string(),
            oper_reason: oper_reason.map(|s| s.to_string())
        };
        server.node().submit_event(new_kline.id, details::NewNetworkBan{ data: new_kline });
    }
    else
    {
        cmd.notice("Invalid kline mask");
    }

    Ok(())
}

