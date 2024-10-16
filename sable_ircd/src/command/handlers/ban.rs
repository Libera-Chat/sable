use super::*;
use sable_network::{chert, network::ban::*};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NewBanAction {
    RefuseConnection,
    RequireSasl,
}

#[derive(Debug, Deserialize)]
struct NewBanArguments {
    #[serde(rename = "type")]
    match_type: Option<BanMatchType>,
    action: Option<NewBanAction>,
    apply_existing: Option<bool>,
    pattern: String,
    duration: i64,
    reason: String,
    oper_reason: Option<String>,
}

#[command_handler("BAN")]
fn handle_ban(
    server: &ClientServer,
    source: UserSource,
    response: &dyn CommandResponse,
    new_ban_str: &str,
) -> CommandResult {
    server.policy().require_oper(&source)?;

    let new_ban_details: NewBanArguments = match serde_json::from_str(new_ban_str) {
        Ok(ban) => ban,
        Err(e) => {
            response.send(message::Fail::new("BAN", "INVALID_BAN", "", &e.to_string()));
            return Ok(());
        }
    };

    let match_type = new_ban_details
        .match_type
        .unwrap_or(BanMatchType::PreRegistration);

    let action = match match_type {
        BanMatchType::PreSasl => {
            // Only valid action here is DenySasl
            NetworkBanAction::DenySasl
        }
        _ => match new_ban_details.action {
            Some(NewBanAction::RefuseConnection) => {
                NetworkBanAction::RefuseConnection(new_ban_details.apply_existing.unwrap_or(true))
            }
            Some(NewBanAction::RequireSasl) => {
                NetworkBanAction::RequireSasl(new_ban_details.apply_existing.unwrap_or(true))
            }
            None => NetworkBanAction::RefuseConnection(true),
        },
    };

    let pattern_parsed = match match_type {
        BanMatchType::PreRegistration => {
            chert::parse::<PreRegistrationBanSettings>(&new_ban_details.pattern)
                .map(|ast| ast.into_root())
        }
        BanMatchType::NewConnection => {
            chert::parse::<NewConnectionBanSettings>(&new_ban_details.pattern)
                .map(|ast| ast.into_root())
        }
        BanMatchType::PreSasl => {
            chert::parse::<PreSaslBanSettings>(&new_ban_details.pattern).map(|ast| ast.into_root())
        }
    };

    let pattern = match pattern_parsed {
        Ok(node) => node,
        Err(e) => {
            response.send(message::Fail::new(
                "BAN",
                "INVALID_BAN_PATTERN",
                "",
                &format!("{:?}", e),
            ));
            return Ok(());
        }
    };

    let timestamp = sable_network::utils::now();
    let expires = timestamp + new_ban_details.duration * 60;

    let new_ban_id: NetworkBanId = server.ids().next();

    let new_ban = event::details::NewNetworkBan {
        match_type,
        pattern,
        action,
        timestamp,
        expires,
        reason: new_ban_details.reason,
        oper_reason: new_ban_details.oper_reason,
        setter_info: source.nuh(),
    };

    server.node().submit_event(new_ban_id, new_ban);

    Ok(())
}
