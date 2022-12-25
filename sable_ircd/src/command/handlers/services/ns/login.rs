use sable_network::{rpc::RemoteServerResponse, network::event::UserLogin};

use super::*;

#[command_handler("LOGIN", in("NS"))]
#[command_handler("IDENTIFY", in("NS"))]
#[command_handler("ID", in("NS"))]
async fn handle_login(net: &Network, source: UserSource<'_>, cmd: &dyn Command,
                      services: ServicesTarget<'_>,
                      mut args: ArgList<'_>) -> CommandResult
{
    let (target_account, password) = match args.len()
    {
        0 => { return Err(CommandError::NotEnoughParameters); }
        1 => (net.account_by_name(&source.nick())?, args.next::<&str>()?),
        _ => (args.next::<wrapper::Account>()?, args.next::<&str>()?)
    };

    let login_request = rpc::RemoteServerRequestType::UserLogin(target_account.id(), password.to_string());
    let login_result = services.send_remote_request(login_request).await;

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