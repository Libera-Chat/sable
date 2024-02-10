use sable_network::{
    network::ban::*,
    rpc::{RemoteServerRequestType, RemoteServerResponse},
};

use super::*;
use base64::prelude::*;

#[command_handler("AUTHENTICATE")]
async fn handle_authenticate(
    source: PreClientSource,
    net: &Network,
    cmd: &dyn Command,
    response: &dyn CommandResponse,
    server: &ClientServer,
    services: Conditional<ServicesTarget<'_>>,
    text: &str,
) -> CommandResult {
    let authenticate_request = if let Some(session) = source.sasl_session.get() {
        // A session already exists, so the argument is "*" or base64-encoded session data
        if text == "*" {
            RemoteServerRequestType::AbortAuthenticate(*session)
        } else {
            let Ok(data) = BASE64_STANDARD.decode(text) else {
                response.notice("Invalid base64");
                return Ok(());
            };

            RemoteServerRequestType::Authenticate(*session, data)
        }
    } else {
        // No session, so the argument is the mechanism name
        // First check whether they're allowed to use SASL
        let user_details = PreSaslBanSettings {
            ip: cmd.connection().remote_addr(),
            tls: cmd.connection().connection.is_tls(),
            mechanism: text.to_owned(),
        };

        for ban in net.network_bans().find_pre_sasl(&user_details) {
            if let NetworkBanAction::DenySasl = ban.action {
                response.numeric(make_numeric!(SaslFail));
                return Ok(());
            }
        }

        // Special case for EXTERNAL, which we can handle without going to services
        if text == "EXTERNAL" {
            return do_sasl_external(source, cmd.connection(), net, response);
        }

        let mechanism = text.to_owned();

        let session = server.ids().next_sasl_session();
        source.sasl_session.set(session).ok();

        RemoteServerRequestType::BeginAuthenticate(session, mechanism)
    };

    match services
        .require()?
        .send_remote_request(authenticate_request)
        .await
    {
        Ok(RemoteServerResponse::Authenticate(status)) => {
            use rpc::AuthenticateStatus::*;

            match status {
                InProgress(data) => {
                    let client_data = if data.is_empty() {
                        "+".to_string()
                    } else {
                        BASE64_STANDARD.encode(data)
                    };
                    response.send(message::Authenticate::new(&client_data));
                }
                Success(account_id) => {
                    source.sasl_account.set(account_id).ok();

                    match net.account(account_id) {
                        Ok(account) => response.numeric(make_numeric!(LoggedIn, &account.name())),
                        Err(err) => tracing::error!(
                            "Successfully logged in to non-existant account {:?}: {:?}",
                            account_id,
                            err
                        ),
                    }
                    response.numeric(make_numeric!(SaslSuccess));
                }
                Fail => {
                    response.numeric(make_numeric!(SaslFail));
                }
                Aborted => {
                    response.numeric(make_numeric!(SaslAborted));
                }
            }
        }
        _ => {
            response.numeric(make_numeric!(SaslAborted));
        }
    }
    Ok(())
}

fn do_sasl_external(
    source: PreClientSource,
    conn: &ClientConnection,
    net: &Network,
    response: &dyn CommandResponse,
) -> CommandResult {
    if let Some(fp) = conn.tls_info().and_then(|ti| ti.fingerprint.as_ref()) {
        if let Some(account) = net.account_with_fingerprint(fp.as_str()) {
            source.sasl_account.set(account.id()).ok();
            response.numeric(make_numeric!(SaslSuccess));
            return Ok(());
        }
    }

    response.numeric(make_numeric!(SaslFail));
    Ok(())
}
