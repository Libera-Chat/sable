use sable_network::{
    network::state::ChannelRoleName,
    policy::RegistrationPolicyService,
    rpc::{RemoteServerResponse, RemoteServicesServerRequestType, RemoteServicesServerResponse},
};

use super::*;

#[command_handler("ACCESS", in("CS"))]
async fn handle_access(
    source: LoggedInUserSource<'_>,
    cmd: &dyn Command,
    services_target: ServicesTarget<'_>,
    channel: wrapper::ChannelRegistration<'_>,
    subcommand: Option<&str>,
    mut args: ArgList<'_>,
) -> CommandResult {
    if let Some(subcommand) = subcommand.map(|s| s.to_ascii_uppercase()) {
        match subcommand.as_ref() {
            "DELETE" => access_delete(source, cmd, services_target, channel, args.next()?).await,
            "SET" => {
                access_modify(
                    source,
                    cmd,
                    services_target,
                    channel,
                    args.next()?,
                    args.next()?,
                )
                .await
            }
            _ => {
                cmd.notice("Syntax: CS ACCESS <#channel> [SET <account> <role>|DELETE <account>]");
                Ok(())
            }
        }
    } else {
        access_list(source, cmd, channel).await
    }
}

async fn access_list(
    source: LoggedInUserSource<'_>,
    cmd: &dyn Command,
    chan: wrapper::ChannelRegistration<'_>,
) -> CommandResult {
    cmd.server()
        .node()
        .policy()
        .can_view_access(&source.user, &chan)?;

    cmd.notice(format_args!("Access list for {}", chan.name()));
    cmd.notice(" ");

    for access in chan.access_entries() {
        cmd.notice(format_args!(
            "{} {}",
            access.user()?.name(),
            access.role()?.name()
        ))
    }

    Ok(())
}

async fn access_modify(
    source: LoggedInUserSource<'_>,
    cmd: &dyn Command,
    services_target: ServicesTarget<'_>,
    chan: wrapper::ChannelRegistration<'_>,
    target_account: wrapper::Account<'_>,
    new_role_name: ChannelRoleName,
) -> CommandResult {
    let Some(new_role) = chan.role_named(&new_role_name) else {
        cmd.notice(format_args!("Role {} does not exist", new_role_name));
        return Ok(());
    };

    cmd.server()
        .node()
        .policy()
        .can_change_access_for(&source.account, &chan, &target_account)?;
    cmd.server()
        .node()
        .policy()
        .can_grant_role(&source.account, &chan, &new_role)?;

    let target_access_id = ChannelAccessId::new(source.account.id(), chan.id());

    let request = RemoteServicesServerRequestType::ModifyAccess {
        source: source.account.id(),
        id: target_access_id,
        role: Some(new_role.id()),
    }
    .into();
    let registration_response = cmd
        .server()
        .node()
        .sync_log()
        .send_remote_request(services_target.into(), request)
        .await;

    tracing::debug!(?registration_response, "Got registration response");
    match registration_response {
        Ok(RemoteServerResponse::Success) => {
            cmd.notice("Access successfully updated");
        }
        Ok(RemoteServerResponse::Services(RemoteServicesServerResponse::AccessDenied)) => {
            cmd.notice("Access denied");
        }
        Ok(response) => {
            tracing::error!(
                ?response,
                "Unexpected response updating channel access in {}",
                chan.name()
            );
            cmd.notice("Error updating access");
        }
        Err(error) => {
            tracing::error!(?error, "Error updating channel access in {}", chan.name());
            cmd.notice("Error updating access");
        }
    }

    Ok(())
}

async fn access_delete(
    source: LoggedInUserSource<'_>,
    cmd: &dyn Command,
    services_target: ServicesTarget<'_>,
    chan: wrapper::ChannelRegistration<'_>,
    target_account: wrapper::Account<'_>,
) -> CommandResult {
    cmd.server()
        .node()
        .policy()
        .can_change_access_for(&source.account, &chan, &target_account)?;

    let Some(target_access) = source.account.has_access_in(chan.id()) else {
        cmd.notice(format_args!(
            "{} does not have access in {}",
            target_account.name(),
            chan.name()
        ));
        return Ok(());
    };

    let request = RemoteServicesServerRequestType::ModifyAccess {
        source: source.account.id(),
        id: target_access.id(),
        role: None,
    }
    .into();
    let registration_response = services_target.send_remote_request(request).await;

    tracing::debug!(?registration_response, "Got registration response");
    match registration_response {
        Ok(RemoteServerResponse::Success) => {
            cmd.notice("Access successfully updated");
        }
        Ok(RemoteServerResponse::Services(RemoteServicesServerResponse::AccessDenied)) => {
            cmd.notice("Access denied");
        }
        Ok(response) => {
            tracing::error!(?response, "Unexpected response updating channel access");
            cmd.notice("Error updating access");
        }
        Err(error) => {
            tracing::error!(?error, "Error updating channel access");
            cmd.notice("Error updating access");
        }
    }

    Ok(())
}
