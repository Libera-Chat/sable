use sable_network::rpc::{RemoteServerRequestType, RemoteServerResponse};

use super::*;
use base64::prelude::*;

#[command_handler("AUTHENTICATE")]
async fn handle_authenticate(source: PreClientSource, cmd: &dyn Command, server: &ClientServer, services: ServicesTarget<'_>,
                             text: &str) -> CommandResult
{
    let authenticate_request = if let Some(session) = source.sasl_session.get()
    {
        // A session already exists, so the argument is "*" or base64-encoded session data
        if text == "*"
        {
            RemoteServerRequestType::AbortAuthenticate(*session)
        }
        else
        {
            let Ok(data) = BASE64_STANDARD.decode(text) else {
                cmd.notice("Invalid base64");
                return Ok(())
            };

            RemoteServerRequestType::Authenticate(*session, data)
        }
    }
    else
    {
        // No session, so the argument is the mechanism name
        let mechanism = text.to_owned();

        let session = server.ids().next_sasl_session();
        source.sasl_session.set(session).ok();

        RemoteServerRequestType::BeginAuthenticate(session, mechanism)
    };

    match services.send_remote_request(authenticate_request).await
    {
        Ok(RemoteServerResponse::Authenticate(status)) =>
        {
            use rpc::AuthenticateStatus::*;

            match status
            {
                InProgress(data) =>
                {
                    let client_data = if data.is_empty() {
                        "+".to_string()
                    } else {
                        BASE64_STANDARD.encode(data)
                    };
                    cmd.response(&message::Authenticate::new(&client_data));
                }
                Success(account) =>
                {
                    source.sasl_account.set(account).ok();

                    cmd.numeric(make_numeric!(SaslSuccess));
                }
                Fail =>
                {
                    cmd.numeric(make_numeric!(SaslFail));
                }
                Aborted =>
                {
                    cmd.numeric(make_numeric!(SaslAborted));
                }
            }
        }
        _ =>
        {
            cmd.numeric(make_numeric!(SaslAborted));
        }
    }
    Ok(())
}