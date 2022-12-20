use sable_network::{
    rpc::{RemoteServerResponse, RemoteServerRequestType},
    network::state::ChannelAccessFlag,
};

use super::*;

pub(super) async fn handle_access(source: &wrapper::User<'_>, cmd: &ClientCommand) -> CommandResult
{
    let subcommand = cmd.args.get(2).map(|s| s.to_ascii_uppercase());

    match (cmd.args.len(), subcommand.as_ref().map(AsRef::as_ref))
    {
        (2, _) => access_list(source, cmd).await,
        (4, Some("DELETE")) => access_delete(source, cmd).await,
        (5, Some("SET")) => access_modify(source, cmd).await,
        _ => {
            cmd.notice("Syntax: CS ACCESS <#channel> [SET <account> <role>|DELETE <account>]");
            Ok(())
        }
    }
}

async fn access_list(user: &wrapper::User<'_>, cmd: &ClientCommand) -> CommandResult
{
    let net = cmd.server.network();

    let Ok(Some(account)) = user.account() else {
        cmd.notice("You are not logged in");
        return Ok(())
    };

    let Ok(channel_name) = ChannelName::from_str(&cmd.args[1]) else {
        cmd.notice(format_args!("Invalid channel name {}", &cmd.args[1]));
        return Ok(())
    };

    let Ok(registration) = net.channel_registration_by_name(channel_name) else {
        cmd.notice(format_args!("{} is not registered", &channel_name));
        return Ok(())
    };

    let Some(source_access) = account.has_access_in(registration.id()) else {
        cmd.notice("Access denied");
        return Ok(())
    };

    if ! source_access.role()?.flags().is_set(ChannelAccessFlag::AccessView)
    {
        cmd.notice("Access denied");
        return Ok(())
    }

    cmd.notice(format_args!("Access list for {}", channel_name));
    cmd.notice(" ");

    for access in registration.access_entries()
    {
        cmd.notice(format_args!("{} {}", access.user()?.name(), access.role()?.name()))
    }

    Ok(())
}

async fn access_modify(user: &wrapper::User<'_>, cmd: &ClientCommand) -> CommandResult
{
    let net = cmd.server.network();

    let Ok(Some(account)) = user.account() else {
        cmd.notice("You are not logged in");
        return Ok(())
    };

    let Ok(channel_name) = ChannelName::from_str(&cmd.args[1]) else {
        cmd.notice(format_args!("Invalid channel name {}", &cmd.args[1]));
        return Ok(())
    };

    let Ok(registration) = net.channel_registration_by_name(channel_name) else {
        cmd.notice(format_args!("{} is not registered", &channel_name));
        return Ok(())
    };

    let Some(source_access) = account.has_access_in(registration.id()) else {
        cmd.notice("Access denied");
        return Ok(())
    };

    if ! source_access.role()?.flags().is_set(ChannelAccessFlag::AccessEdit)
    {
        cmd.notice("Access denied");
        return Ok(())
    }

    let Ok(target_accountname) = Nickname::from_str(&cmd.args[3]) else {
        cmd.notice(format_args!("Invalid account name {}", &cmd.args[3]));
        return Ok(())
    };

    let Ok(target_account) = net.account_by_name(target_accountname) else {
        cmd.notice(format_args!("{} is not registered", target_accountname));
        return Ok(())
    };

    let target_access_id = ChannelAccessId::new(target_account.id(), registration.id());

    if let Some(current_flags) = net.channel_access(target_access_id)
                                    .ok()
                                    .and_then(|access| access.role().ok().map(|r| r.flags()))
    {
        if ! source_access.role()?.flags().dominates(&current_flags)
        {
            cmd.notice("Access denied");
            return Ok(())
        }
    }

    let Ok(new_role_name) = cmd.args[4].parse() else {
        cmd.notice(format_args!("Invalid role name {}", cmd.args[4]));
        return Ok(())
    };

    let Some(new_role) = registration.role_named(&new_role_name) else {
        cmd.notice(format_args!("Role {} does not exist", new_role_name));
        return Ok(())
    };

    if ! source_access.role()?.flags().dominates(&new_role.flags())
    {
        cmd.notice("Access denied");
        return Ok(())
    }

    let Some(services_target) = net.current_services() else {
        cmd.notice("Services are currently unavailable");
        return Ok(())
    };

    let request = RemoteServerRequestType::ModifyAccess { source: account.id(), id: target_access_id, role: Some(new_role.id()) };
    let registration_response = cmd.server.server().sync_log().send_remote_request(services_target, request).await;

    tracing::debug!(?registration_response, "Got registration response");
    match registration_response
    {
        Ok(RemoteServerResponse::Success) =>
        {
            cmd.notice("Access successfully updated");
        }
        Ok(RemoteServerResponse::AccessDenied) =>
        {
            cmd.notice("Access denied");
        }
        Ok(response) =>
        {
            tracing::error!(?response, ?channel_name, "Unexpected response updating channel access");
            cmd.notice("Error updating access");
        }
        Err(error) =>
        {
            tracing::error!(?error, ?channel_name, "Error updating channel access");
            cmd.notice("Error updating access");
        }
    }

    Ok(())
}

async fn access_delete(user: &wrapper::User<'_>, cmd: &ClientCommand) -> CommandResult
{
    let net = cmd.server.network();

    let Ok(Some(account)) = user.account() else {
        cmd.notice("You are not logged in");
        return Ok(())
    };

    let Ok(channel_name) = ChannelName::from_str(&cmd.args[1]) else {
        cmd.notice(format_args!("Invalid channel name {}", &cmd.args[1]));
        return Ok(())
    };

    let Ok(registration) = net.channel_registration_by_name(channel_name) else {
        cmd.notice(format_args!("{} is not registered", &channel_name));
        return Ok(())
    };

    let Some(source_access) = account.has_access_in(registration.id()) else {
        cmd.notice("Access denied");
        return Ok(())
    };

    if ! source_access.role()?.flags().is_set(ChannelAccessFlag::AccessEdit)
    {
        cmd.notice("Access denied");
        return Ok(())
    }

    let Ok(target_accountname) = Nickname::from_str(&cmd.args[2]) else {
        cmd.notice(format_args!("Invalid account name {}", &cmd.args[2]));
        return Ok(())
    };

    let Ok(target_account) = net.account_by_name(target_accountname) else {
        cmd.notice(format_args!("{} is not registered", target_accountname));
        return Ok(())
    };

    let target_access_id = ChannelAccessId::new(target_account.id(), registration.id());

    if let Some(current_flags) = net.channel_access(target_access_id)
                                    .ok()
                                    .and_then(|access| access.role().ok().map(|r| r.flags()))
    {
        if ! source_access.role()?.flags().dominates(&current_flags)
        {
            cmd.notice("Access denied");
            return Ok(())
        }
    }

    let Some(services_target) = net.current_services() else {
        cmd.notice("Services are currently unavailable");
        return Ok(())
    };

    let request = RemoteServerRequestType::ModifyAccess { source: account.id(), id: target_access_id, role: None };
    let registration_response = cmd.server.server().sync_log().send_remote_request(services_target, request).await;

    tracing::debug!(?registration_response, "Got registration response");
    match registration_response
    {
        Ok(RemoteServerResponse::Success) =>
        {
            cmd.notice("Access successfully updated");
        }
        Ok(RemoteServerResponse::AccessDenied) =>
        {
            cmd.notice("Access denied");
        }
        Ok(response) =>
        {
            tracing::error!(?response, ?channel_name, "Unexpected response updating channel access");
            cmd.notice("Error updating access");
        }
        Err(error) =>
        {
            tracing::error!(?error, ?channel_name, "Error updating channel access");
            cmd.notice("Error updating access");
        }
    }

    Ok(())
}