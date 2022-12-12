use sable_network::{
    rpc::{RemoteServerResponse, RemoteServerRequestType}
};

use super::*;

pub(super) async fn handle_access(source: UserId, cmd: Arc<ClientCommand>) -> CommandResult
{
    match cmd.args.len()
    {
        2 => access_list(source, cmd).await,
        4.. => access_modify(source, cmd).await,
        _ => {
            cmd.notice("Syntax: CS ACCESS <#channel> [account] [flags]");
            Ok(())
        }
    }
}

async fn access_list(_source: UserId, cmd: Arc<ClientCommand>) -> CommandResult
{
    let net = cmd.server.network();

    let Ok(channel_name) = ChannelName::from_str(&cmd.args[1]) else {
        cmd.notice(format_args!("Invalid channel name {}", &cmd.args[1]));
        return Ok(())
    };

    let Ok(registration) = net.channel_registration_by_name(channel_name) else {
        cmd.notice(format_args!("{} is not registered", &channel_name));
        return Ok(())
    };

    cmd.notice(format_args!("Access list for {}", channel_name));
    cmd.notice(" ");

    for access in registration.access_entries()
    {
        cmd.notice(format_args!("{} +{}", access.user()?.name(), access.flags().to_chars()))
    }

    Ok(())
}

fn parse_access_change(existing: Option<ChannelAccessSet>, change_string: &str) -> Result<Option<ChannelAccessSet>, ()>
{
    let mut chars = change_string.chars();
    let first = chars.next();

    let mut change = ChannelAccessSet::new();
    for char in chars
    {
        if let Some(flag) = ChannelAccessSet::flag_for(char)
        {
            change |= flag;
        }
        else
        {
            return Err(());
        }
    }

    let existing = existing.unwrap_or(ChannelAccessSet::new());

    let new = match first
    {
        Some('+') => existing | change,
        Some('-') => existing & !change,
        Some('=') => change,
        _ => { return Err(()); }
    };

    Ok(if new.is_empty() { None } else { Some(new) })
}

async fn access_modify(source: UserId, cmd: Arc<ClientCommand>) -> CommandResult
{
    let net = cmd.server.network();

    let user = net.user(source)?;

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

    if ! source_access.flags().is_set(ChannelAccessFlag::Access)
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

    let current_flags = net.channel_access(target_access_id).ok().map(|access| access.flags());

    let Ok(new_flags) = parse_access_change(current_flags, &cmd.args[3]) else {
        cmd.notice(format_args!("Invalid flag string {}", cmd.args[3]));
        return Ok(())
    };

    let Some(services_target) = net.current_services() else {
        cmd.notice("Services are currently unavailable");
        return Ok(())
    };

    let request = RemoteServerRequestType::ModifyAccess { source: account.id(), id: target_access_id, flags: new_flags };
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