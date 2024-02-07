use super::*;
use sable_network::chert;
use sable_network::network::ban::*;

const DEFAULT_KLINE_DURATION: u32 = 1440;

#[command_handler("KLINE")]
fn handle_kline(
    server: &ClientServer,
    response: &dyn CommandResponse,
    source: UserSource,
    audit: AuditLogger,
    duration: IfParses<u32>,
    mask: &str,
    message: &str,
) -> CommandResult {
    server.policy().require_oper(&source)?;

    let duration = duration.unwrap_or(DEFAULT_KLINE_DURATION) as i64;

    let parts: Vec<_> = message.splitn(2, '|').collect();
    let user_reason = parts[0];
    let oper_reason = parts.get(1);

    let mask_parts: Vec<_> = mask.split('@').collect();

    if let [user, host] = mask_parts[..] {
        let user_condition = if user == "*" {
            None
        } else {
            Some(format!("user == \"{}\"", user))
        };

        let host_condition = if host.parse::<std::net::IpAddr>().is_ok() {
            format!("ip == {}", host)
        } else if let Some((first, second)) = host.rsplit_once('/') {
            if second.parse::<u8>().is_ok() && first.parse::<std::net::IpAddr>().is_ok() {
                format!("ip in {}", host)
            } else {
                format!("host == \"{}\"", host)
            }
        } else {
            format!("host == \"{}\"", host)
        };

        let condition = if let Some(user_condition) = user_condition {
            format!("{} && {}", user_condition, host_condition)
        } else {
            host_condition
        };

        let pattern = match chert::parse::<PreRegistrationBanSettings>(&condition) {
            Err(err) => {
                tracing::error!(condition, ?err, "Translated ban condition failed to parse");
                response.send(message::Notice::new(
                    server,
                    &source,
                    "Internal error: condition failed to parse",
                ));
                return Ok(());
            }
            Ok(parsed) => parsed.get_root().clone(),
        };

        audit
            .ban()
            .target_str(condition)
            .target_duration(duration)
            .reason(message.to_string())
            .log();

        let new_kline = event::NewNetworkBan {
            pattern,
            setter_info: source.0.nuh(),
            timestamp: sable_network::utils::now(),
            expires: sable_network::utils::now() + (duration * 60),
            reason: user_reason.to_string(),
            oper_reason: oper_reason.map(|s| s.to_string()),
        };
        server
            .node()
            .submit_event(server.ids().next_network_ban(), new_kline);
    } else {
        response.notice("Invalid kline mask");
    }

    Ok(())
}
