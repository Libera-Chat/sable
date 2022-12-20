use sable_network::{rpc::RemoteServerResponse, prelude::event::UserLogin};

use super::*;

pub(super) async fn handle_login(source: &wrapper::User<'_>, cmd: &ClientCommand) -> CommandResult
{
    let net = cmd.server.network();

    let (username, password) = match cmd.args.len()
    {
        0|1 => {
            cmd.notice("Not enough arguments");
            return Ok(())
        }
        2 => {
            (source.nick(), cmd.args[1].clone())
        }
        _ => {
            (Nickname::from_str(&cmd.args[1])?, cmd.args[2].clone())
        }
    };

    let Ok(target_account) = net.account_by_name(username) else {
        let msg = format!("{} is not a registered account", username);
        cmd.notice(msg);
        return Ok(());
    };

    let Some(services_target) = net.current_services() else {
        cmd.notice("Services are currently unavailable");
        return Ok(())
    };

    let login_request = rpc::RemoteServerRequestType::UserLogin(target_account.id(), password);
    let login_result = cmd.server.server().sync_log().send_remote_request(services_target, login_request).await;

    match login_result
    {
        Ok(RemoteServerResponse::LogUserIn(account)) => {
            if account == target_account.id()
            {
                cmd.new_event(source.id(), UserLogin { account: Some(account) });
                let msg = format!("You are now logged in to {}", target_account.name());
                cmd.notice(msg);
            }
            else
            {
                tracing::error!("Got login success for mismatched account? {:?}/{:?}", target_account.id(), account);
                cmd.notice("Login failed (internal error)");
            }
        }
        Ok(RemoteServerResponse::InvalidCredentials) => {
            let msg = format!("Invalid credentials for {}", target_account.name());
            cmd.notice(msg);
        }
        _ => {
            tracing::error!("Got unexpected response to login request: {:?}", login_result);
            cmd.notice("Login failed (internal error)");
        }
    }

    Ok(())
}