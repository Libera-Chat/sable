use sable_network::{
    rpc::{RemoteServerResponse, RemoteServerRequestType}
};

use super::*;

pub(super) async fn handle_register(user: &wrapper::User<'_>, cmd: &ClientCommand) -> CommandResult
{
    let net = cmd.server.network();

    let Ok(Some(account)) = user.account() else {
        cmd.notice("You must be logged in before registering a channel");
        return Ok(())
    };

    let Some(channel_name) = cmd.args.get(1) else {
        cmd.notice("You need to specify a channel name to register");
        return Ok(())
    };

    let Ok(channel_name) = ChannelName::from_str(&channel_name) else {
        cmd.notice(format_args!("Invalid channel name {}", channel_name));
        return Ok(())
    };

    let Ok(channel) = net.channel_by_name(&channel_name) else {
        cmd.notice(format_args!("Channel {} does not exist", channel_name));
        return Ok(())
    };

    if net.channel_registration_by_name(channel_name).is_ok()
    {
        cmd.notice(format_args!("Channel {} is already registered", channel_name));
        return Ok(())
    }

    let Some(services_target) = net.current_services() else {
        cmd.notice("Services are currently unavailable");
        return Ok(())
    };

    let request = RemoteServerRequestType::RegisterChannel(account.id(), channel.id());
    let registration_response = cmd.server.server().sync_log().send_remote_request(services_target, request).await;

    tracing::debug!(?registration_response, "Got registration response");
    match registration_response
    {
        Ok(RemoteServerResponse::Success) =>
        {
            cmd.notice(format_args!("Channel {} successfully registered", channel.name()));
        }
        Ok(RemoteServerResponse::AlreadyExists) =>
        {
            cmd.notice(format_args!("Channel {} is already registered", channel.name()));
        }
        Ok(response) =>
        {
            tracing::error!(?response, ?channel_name, "Unexpected response registering channel");
            cmd.notice(format_args!("Channel {} could not be registered", channel.name()));
        }
        Err(error) =>
        {
            tracing::error!(?error, ?channel_name, "Error registering channel");
            cmd.notice(format_args!("Channel {} could not be registered", channel.name()));
        }
    }
    Ok(())
}