use super::*;

//#[command_handler("REGISTER")]

mod reg_handle_register {
    use super::CommandContext;

    fn call_proxy(ctx: crate::ClientCommand) -> Option<crate::command::AsyncHandler>
    {
        Some(Box::pin(async move {
            let ctx = ctx;
            crate::command::plumbing::call_handler_async(&ctx, &super::handle_register, &ctx.args).await;
        }))
    }
    inventory::submit!(crate::command::CommandRegistration {
        command: "REGISTER",
        handler: call_proxy
    });
}


pub async fn handle_register(server: &ClientServer, network: &Network, source: CommandSource<'_>, cmd: &ClientCommand,
                         account: &str, email: &str, password: &str) -> CommandResult
{
    match source
    {
        CommandSource::PreClient(_) =>
        {
            cmd.response(&message::Fail::new("REGISTER",
                                             "COMPLETE_CONNECTION_REQUIRED",
                                             "*",
                                             "Finish connecting before registering"));
            Ok(())
        }
        CommandSource::User(user) => do_register_user(server, network, user, cmd, account, email, password).await
    }
}

async fn do_register_user(server: &ClientServer, network: &Network, source: wrapper::User<'_>, cmd: &ClientCommand,
                    account: &str, _email: &str, password: &str) -> CommandResult
{
    let Some(services_name) = network.current_services() else {
        cmd.connection.send(&message::Fail::new("REGISTER",
                                                "TEMPORARILY_UNAVAILABLE",
                                                "*",
                                                "Services are temporarily unavailable"));
        return Ok(())
    };

    let requested_account = if account == "*" { source.nick() } else { Nickname::from_str(account)? };

    if requested_account != source.nick()
    {
        // We don't support registering with an account other than your current nick (yet?)
        cmd.response(&message::Fail::new("REGISTER",
                                            "ACCOUNT_NAME_MUST_BE_NICK",
                                            account,
                                            "Your account name must be your current nickname"));
        return Ok(())
    }

    if network.account_by_name(requested_account).is_ok()
    {
        cmd.connection.send(&message::Fail::new("REGISTER",
                                                "ACCOUNT_EXISTS",
                                                requested_account.value().as_str(),
                                                "Account already exists"));
        return Ok(())
    }

    let message = rpc::RemoteServerRequestType::RegisterUser(requested_account, password.to_owned());

    match cmd.server.server().sync_log().send_remote_request(services_name, message).await
    {
        Ok(rpc::RemoteServerResponse::LogUserIn(account)) =>
        {
            cmd.server.add_action(CommandAction::state_change(source.id(), event::UserLogin {
                account: Some(account)
            }));
            cmd.connection.send(&message::Register::new("SUCCESS", requested_account, "You have successfully registered"));
        }
        Ok(rpc::RemoteServerResponse::AlreadyExists) =>
        {
            cmd.connection.send(&message::Fail::new("REGISTER",
                                                    "ACCOUNT_EXISTS",
                                                    account,
                                                    "Account already exists"));
        }
        Ok(response) =>
        {
            tracing::error!(?response, "Unexpected response from services");
            cmd.connection.send(&message::Fail::new("REGISTER",
                                                    "TEMPORARILY_UNAVAILABLE",
                                                    account,
                                                    "Services are temporarily unavailable"));
        }
        Err(e) =>
        {
            tracing::error!(?e, "Error sending register request");
            cmd.connection.send(&message::Fail::new("REGISTER",
                                                    "TEMPORARILY_UNAVAILABLE",
                                                    account,
                                                    "Services are temporarily unavailable"));
        }
    }

    Ok(())
}
