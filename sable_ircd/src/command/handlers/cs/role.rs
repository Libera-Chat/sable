use sable_network::{
    rpc::{RemoteServerResponse, RemoteServerRequestType},
    network::state::{ChannelAccessFlag, ChannelRoleName},
};

use super::*;

pub(super) async fn handle_role(source: &wrapper::User<'_>, cmd: &ClientCommand) -> CommandResult
{
    let subcommand = cmd.args.get(2).map(|s| s.to_ascii_uppercase());

    match (cmd.args.len(), subcommand.as_ref().map(AsRef::as_ref))
    {
        (2, _) => role_list(&source, cmd).await,
        (5.., Some("ADD")) => role_add(&source, cmd).await,
        (4, Some("DELETE")) => role_delete(&source, cmd).await,
        (5.., Some("EDIT")) => role_edit(&source, cmd).await,
        _ => {
            cmd.notice("Syntax: CS ROLE <#channel> [ADD <name> <flags...> | EDIT <name> +add_flags -remove_flags | DELETE <name>]");
            Ok(())
        }
    }
}

async fn role_list(user: &wrapper::User<'_>, cmd: &ClientCommand) -> CommandResult
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

    if ! source_access.role()?.flags().is_set(ChannelAccessFlag::RoleView)
    {
        cmd.notice("Access denied");
        return Ok(())
    }

    cmd.notice(format_args!("Role list for {}", channel_name));
    cmd.notice(" ");

    for role in registration.roles()
    {
        cmd.notice(format_args!("{} {}", role.name(), state::HumanReadableChannelAccessSet::from(role.flags())))
    }

    Ok(())
}

async fn role_edit(user: &wrapper::User<'_>, cmd: &ClientCommand) -> CommandResult
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

    let source_flags = source_access.role()?.flags();

    if ! source_flags.is_set(ChannelAccessFlag::RoleEdit)
    {
        cmd.notice("Access denied");
        return Ok(())
    }

    let Ok(target_role_name) = ChannelRoleName::from_str(&cmd.args[3]) else {
        cmd.notice(format_args!("Invalid role name {}", &cmd.args[3]));
        return Ok(())
    };

    let Some(target_role) = registration.role_named(&target_role_name) else {
        cmd.notice(format_args!("No such role {}", target_role_name));
        return Ok(())
    };

    let mut flags = target_role.flags();

    for flag_str in &cmd.args[4..]
    {
        let (adding, flag_name) = match flag_str.as_bytes()[0]
        {
            b'+' => (true, &flag_str[1..]),
            b'-' => (false, &flag_str[1..]),
            _ => (true, &flag_str[..]),
        };

        let Ok(flag) = ChannelAccessFlag::from_str(flag_name) else {
            cmd.notice(format_args!("Invalid access flag {}", flag_name));
            return Ok(());
        };

        if ! source_flags.is_set(flag)
        {
            cmd.notice("Access denied");
            return Ok(())
        }

        if adding
        {
            flags |= flag;
        }
        else
        {
            flags &= !flag;
        }
    }

    let Some(services_target) = net.current_services() else {
        cmd.notice("Services are currently unavailable");
        return Ok(())
    };

    let request = RemoteServerRequestType::ModifyRole { source: account.id(), id: target_role.id(), flags: Some(flags) };
    let registration_response = cmd.server.server().sync_log().send_remote_request(services_target, request).await;

    tracing::debug!(?registration_response, "Got registration response");
    match registration_response
    {
        Ok(RemoteServerResponse::Success) =>
        {
            cmd.notice("Role successfully updated");
        }
        Ok(RemoteServerResponse::AccessDenied) =>
        {
            cmd.notice("Access denied");
        }
        Ok(response) =>
        {
            tracing::error!(?response, ?channel_name, "Unexpected response updating channel access");
            cmd.notice("Error updating role");
        }
        Err(error) =>
        {
            tracing::error!(?error, ?channel_name, "Error updating channel role");
            cmd.notice("Error updating role");
        }
    }

    Ok(())
}

async fn role_add(user: &wrapper::User<'_>, cmd: &ClientCommand) -> CommandResult
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

    let source_flags = source_access.role()?.flags();

    if ! source_flags.is_set(ChannelAccessFlag::RoleEdit)
    {
        cmd.notice("Access denied");
        return Ok(())
    }

    let Ok(target_role_name) = CustomRoleName::from_str(&cmd.args[3]) else {
        cmd.notice(format_args!("Invalid role name {}", &cmd.args[3]));
        return Ok(())
    };

    let mut flags = state::ChannelAccessSet::new();

    for flag_str in &cmd.args[4..]
    {
        let Ok(flag) = ChannelAccessFlag::from_str(flag_str) else {
            cmd.notice(format_args!("Invalid access flag {}", flag_str));
            return Ok(());
        };

        if ! source_flags.is_set(flag)
        {
            cmd.notice("Access denied");
            return Ok(())
        }

        flags |= flag;
    }

    let Some(services_target) = net.current_services() else {
        cmd.notice("Services are currently unavailable");
        return Ok(())
    };

    let request = RemoteServerRequestType::CreateRole { source: account.id(), channel: registration.id(), name: target_role_name, flags: flags };
    let registration_response = cmd.server.server().sync_log().send_remote_request(services_target, request).await;

    tracing::debug!(?registration_response, "Got registration response");
    match registration_response
    {
        Ok(RemoteServerResponse::Success) =>
        {
            cmd.notice("Role successfully updated");
        }
        Ok(RemoteServerResponse::AccessDenied) =>
        {
            cmd.notice("Access denied");
        }
        Ok(response) =>
        {
            tracing::error!(?response, ?channel_name, "Unexpected response updating channel access");
            cmd.notice("Error updating role");
        }
        Err(error) =>
        {
            tracing::error!(?error, ?channel_name, "Error updating channel role");
            cmd.notice("Error updating role");
        }
    }

    Ok(())
}

async fn role_delete(user: &wrapper::User<'_>, cmd: &ClientCommand) -> CommandResult
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

    let source_flags = source_access.role()?.flags();

    if ! source_flags.is_set(ChannelAccessFlag::RoleEdit)
    {
        cmd.notice("Access denied");
        return Ok(())
    }

    let Ok(target_role_name) = ChannelRoleName::from_str(&cmd.args[3]) else {
        cmd.notice(format_args!("Invalid role name {}", &cmd.args[3]));
        return Ok(())
    };

    let Some(target_role) = registration.role_named(&target_role_name) else {
        cmd.notice(format_args!("No such role {}", target_role_name));
        return Ok(())
    };

    if ! source_access.role()?.flags().dominates(&target_role.flags())
    {
        cmd.notice("Access denied");
        return Ok(())
    }

    let Some(services_target) = net.current_services() else {
        cmd.notice("Services are currently unavailable");
        return Ok(())
    };

    let request = RemoteServerRequestType::ModifyRole { source: account.id(), id: target_role.id(), flags: None };
    let registration_response = cmd.server.server().sync_log().send_remote_request(services_target, request).await;

    tracing::debug!(?registration_response, "Got registration response");
    match registration_response
    {
        Ok(RemoteServerResponse::Success) =>
        {
            cmd.notice("Role successfully updated");
        }
        Ok(RemoteServerResponse::AccessDenied) =>
        {
            cmd.notice("Access denied");
        }
        Ok(response) =>
        {
            tracing::error!(?response, ?channel_name, "Unexpected response updating channel role");
            cmd.notice("Error updating role");
        }
        Err(error) =>
        {
            tracing::error!(?error, ?channel_name, "Error updating channel role");
            cmd.notice("Error updating role");
        }
    }

    Ok(())
}