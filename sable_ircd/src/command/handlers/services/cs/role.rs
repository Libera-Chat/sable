use sable_network::{
    network::state::ChannelAccessFlag,
    policy::RegistrationPolicyService,
    rpc::{RemoteServerRequestType, RemoteServerResponse},
};

use super::*;

#[command_handler("ROLE", in("CS"))]
async fn handle_role(
    source: LoggedInUserSource<'_>,
    cmd: &dyn Command,
    services_target: ServicesTarget<'_>,
    channel: wrapper::ChannelRegistration<'_>,
    subcommand: Option<&str>,
    mut args: ArgList<'_>,
) -> CommandResult {
    if let Some(subcommand) = subcommand.map(|s| s.to_ascii_uppercase()) {
        match subcommand.as_ref() {
            "ADD" => role_add(source, cmd, services_target, channel, args.next()?, args).await,
            "DELETE" => role_delete(source, cmd, services_target, channel, args.next()?).await,
            "EDIT" => role_edit(source, cmd, services_target, channel, args.next()?, args).await,
            _ => {
                cmd.notice("Syntax: CS ROLE <#channel> [ADD <name> <flags...> | EDIT <name> +add_flags -remove_flags | DELETE <name>]");
                Ok(())
            }
        }
    } else {
        role_list(source, cmd, channel).await
    }
}

async fn role_list(
    source: LoggedInUserSource<'_>,
    cmd: &dyn Command,
    chan: wrapper::ChannelRegistration<'_>,
) -> CommandResult {
    cmd.server()
        .node()
        .policy()
        .can_view_roles(&source.user, &chan)?;

    cmd.notice(format_args!("Role list for {}", chan.name()));
    cmd.notice(" ");

    for role in chan.roles() {
        cmd.notice(format_args!(
            "{} {}",
            role.name(),
            state::HumanReadableChannelAccessSet::from(role.flags())
        ))
    }

    Ok(())
}

async fn role_edit(
    source: LoggedInUserSource<'_>,
    cmd: &dyn Command,
    services_target: ServicesTarget<'_>,
    chan: wrapper::ChannelRegistration<'_>,
    target_role_name: state::ChannelRoleName,
    mut args: ArgList<'_>,
) -> CommandResult {
    let Some(target_role) = chan.role_named(&target_role_name) else {
        cmd.notice(format_args!("No such role {}", target_role_name));
        return Ok(());
    };

    cmd.server()
        .node()
        .policy()
        .can_edit_role(&source.account, &chan, &target_role)?;

    let mut flags = target_role.flags();

    while let Ok(flag_str) = args.next::<&str>() {
        let (adding, flag_name) = match flag_str.as_bytes()[0] {
            b'+' => (true, &flag_str[1..]),
            b'-' => (false, &flag_str[1..]),
            _ => (true, flag_str),
        };

        let Ok(flag) = ChannelAccessFlag::from_str(flag_name) else {
            cmd.notice(format_args!("Invalid access flag {}", flag_name));
            return Ok(());
        };

        if adding {
            flags |= flag;
        } else {
            flags &= !flag;
        }
    }

    cmd.server()
        .node()
        .policy()
        .can_create_role(&source.account, &chan, &flags)?;

    let request = RemoteServerRequestType::ModifyRole {
        source: source.account.id(),
        id: target_role.id(),
        flags: Some(flags),
    };
    let registration_response = services_target.send_remote_request(request).await;

    tracing::debug!(?registration_response, "Got registration response");
    match registration_response {
        Ok(RemoteServerResponse::Success) => {
            cmd.notice("Role successfully updated");
        }
        Ok(RemoteServerResponse::AccessDenied) => {
            cmd.notice("Access denied");
        }
        Ok(response) => {
            tracing::error!(?response, "Unexpected response updating channel access");
            cmd.notice("Error updating role");
        }
        Err(error) => {
            tracing::error!(?error, "Error updating channel role");
            cmd.notice("Error updating role");
        }
    }

    Ok(())
}

async fn role_add(
    source: LoggedInUserSource<'_>,
    cmd: &dyn Command,
    services_target: ServicesTarget<'_>,
    chan: wrapper::ChannelRegistration<'_>,
    target_role_name: CustomRoleName,
    mut args: ArgList<'_>,
) -> CommandResult {
    let mut flags = state::ChannelAccessSet::new();

    while let Ok(flag_str) = args.next::<&str>() {
        let Ok(flag) = ChannelAccessFlag::from_str(flag_str) else {
            cmd.notice(format_args!("Invalid access flag {}", flag_str));
            return Ok(());
        };

        flags |= flag;
    }

    cmd.server()
        .node()
        .policy()
        .can_create_role(&source.account, &chan, &flags)?;

    let request = RemoteServerRequestType::CreateRole {
        source: source.account.id(),
        channel: chan.id(),
        name: target_role_name,
        flags,
    };
    let registration_response = services_target.send_remote_request(request).await;

    tracing::debug!(?registration_response, "Got registration response");
    match registration_response {
        Ok(RemoteServerResponse::Success) => {
            cmd.notice("Role successfully updated");
        }
        Ok(RemoteServerResponse::AccessDenied) => {
            cmd.notice("Access denied");
        }
        Ok(response) => {
            tracing::error!(?response, "Unexpected response updating channel access");
            cmd.notice("Error updating role");
        }
        Err(error) => {
            tracing::error!(?error, "Error updating channel role");
            cmd.notice("Error updating role");
        }
    }

    Ok(())
}

async fn role_delete(
    source: LoggedInUserSource<'_>,
    cmd: &dyn Command,
    services_target: ServicesTarget<'_>,
    chan: wrapper::ChannelRegistration<'_>,
    target_role_name: state::ChannelRoleName,
) -> CommandResult {
    let Some(target_role) = chan.role_named(&target_role_name) else {
        cmd.notice(format_args!("No such role {}", target_role_name));
        return Ok(());
    };

    cmd.server()
        .node()
        .policy()
        .can_edit_role(&source.account, &chan, &target_role)?;

    let request = RemoteServerRequestType::ModifyRole {
        source: source.account.id(),
        id: target_role.id(),
        flags: None,
    };
    let registration_response = services_target.send_remote_request(request).await;

    tracing::debug!(?registration_response, "Got registration response");
    match registration_response {
        Ok(RemoteServerResponse::Success) => {
            cmd.notice("Role successfully updated");
        }
        Ok(RemoteServerResponse::AccessDenied) => {
            cmd.notice("Access denied");
        }
        Ok(response) => {
            tracing::error!(?response, "Unexpected response updating channel role");
            cmd.notice("Error updating role");
        }
        Err(error) => {
            tracing::error!(?error, "Error updating channel role");
            cmd.notice("Error updating role");
        }
    }

    Ok(())
}
