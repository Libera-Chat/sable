use super::*;
use sable_network::network::ban::*;

const DEFAULT_KLINE_DURATION: u32 = 1440;

#[command_handler("KLINE")]
fn handle_kline(server: &ClientServer, cmd: &dyn Command, source: UserSource, audit: AuditLogger,
                duration: IfParses<u32>, mask: &str, message: &str) -> CommandResult
{
    server.policy().require_oper(&source)?;

    let duration = duration.unwrap_or(DEFAULT_KLINE_DURATION) as i64;

    let parts:Vec<_> = message.splitn(2, '|').collect();
    let user_reason = parts[0];
    let oper_reason = parts.get(1);

    let mask_parts:Vec<_> = mask.split('@').collect();

    if let [user, host] = mask_parts[..]
    {
        audit.ban().target_str(mask.to_string())
                   .target_duration(duration)
                   .reason(message.to_string())
                   .log();

        let matcher = match NetworkBanMatch::from_user_host(user, host) {
            Ok(matcher) => matcher,
            Err(_) => {
                cmd.notice(format_args!("A network ban is already set on {}", mask));
                return Ok(())
            }
        };

        let new_kline = event::NewNetworkBan {
            matcher,
            action: NetworkBanAction::RefuseConnection(true),
            setter_info: source.0.nuh(),
            timestamp: sable_network::utils::now(),
            expires: sable_network::utils::now() + (duration * 60),
            reason: user_reason.to_string(),
            oper_reason: oper_reason.map(|s| s.to_string())
        };
        server.node().submit_event(server.ids().next_network_ban(), new_kline);
    }
    else
    {
        cmd.notice("Invalid kline mask");
    }

    Ok(())
}

