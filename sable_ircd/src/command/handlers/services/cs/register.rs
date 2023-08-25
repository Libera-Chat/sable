use sable_network::rpc::{RemoteServerRequestType, RemoteServerResponse};

use super::*;

#[command_handler("REGISTER", in("CS"))]
async fn handle_register(
    source: LoggedInUserSource<'_>,
    cmd: &dyn Command,
    services_target: ServicesTarget<'_>,
    channel: wrapper::Channel<'_>,
) -> CommandResult {
    if channel.is_registered().is_some() {
        cmd.notice(format_args!(
            "Channel {} is already registered",
            channel.name()
        ));
        return Ok(());
    }

    let request = RemoteServerRequestType::RegisterChannel(source.account.id(), channel.id());
    let registration_response = services_target.send_remote_request(request).await;

    tracing::debug!(?registration_response, "Got registration response");
    match registration_response {
        Ok(RemoteServerResponse::Success) => {
            cmd.notice(format_args!(
                "Channel {} successfully registered",
                channel.name()
            ));
        }
        Ok(RemoteServerResponse::AlreadyExists) => {
            cmd.notice(format_args!(
                "Channel {} is already registered",
                channel.name()
            ));
        }
        Ok(response) => {
            tracing::error!(?response, "Unexpected response registering channel");
            cmd.notice(format_args!(
                "Channel {} could not be registered",
                channel.name()
            ));
        }
        Err(error) => {
            tracing::error!(?error, "Error registering channel");
            cmd.notice(format_args!(
                "Channel {} could not be registered",
                channel.name()
            ));
        }
    }
    Ok(())
}
