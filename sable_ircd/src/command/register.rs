use super::*;

command_handler!("REGISTER" => RegisterHandler {
    fn min_parameters(&self) -> usize { 3 }

    fn handle_preclient(&mut self, _source: &PreClient, cmd: &ClientCommand) -> CommandResult
    {
        cmd.connection.send(&message::Fail::new("REGISTER",
                                                "COMPLETE_CONNECTION_REQUIRED",
                                                "*",
                                                "Finish connecting before registering"));
        Ok(())
    }

    fn handle_user_async<'a>(&mut self, source: UserId, cmd: Arc<ClientCommand>) -> Option<server::AsyncHandler<'a>>
    {
        Some(Box::pin(async move {
            let network = cmd.server.network();

            let Some(services_name) = network.current_services() else {
                cmd.connection.send(&message::Fail::new("REGISTER",
                                                        "TEMPORARILY_UNAVAILABLE",
                                                        "*",
                                                        "Services are temporarily unavailable"));
                return Ok(())
            };

            let Some(account_arg) = cmd.args.get(0) else {
                cmd.connection.send(&message::Fail::new("REGISTER",
                                                        "BAD_ACCOUNT_NAME",
                                                        "*",
                                                        "You need to specify an account name"));
                return Ok(())
            };


            let source_user = network.user(source)?;
            let requested_account = if account_arg == "*" { Ok(source_user.nick()) } else { Nickname::from_str(account_arg) }?;

            if requested_account != source_user.nick()
            {
                // We don't support registering with an account other than your current nick (yet?)
                cmd.connection.send(&message::Fail::new("REGISTER",
                                                        "ACCOUNT_NAME_MUST_BE_NICK",
                                                        account_arg,
                                                        "Your account name must be your current nickname"));
                return Ok(())
            }

            let Some(password) = cmd.args.get(2).cloned() else {
                cmd.connection.send(&message::Fail::new("REGISTER",
                                                        "UNACCEPTABLE_PASSWORD",
                                                        account_arg,
                                                        "You need to specify a password"));
                return Ok(())
            };

            if network.account_by_name(requested_account).is_ok()
            {
                cmd.connection.send(&message::Fail::new("REGISTER",
                                                        "ACCOUNT_EXISTS",
                                                        requested_account.value().as_str(),
                                                        "Account already exists"));
                return Ok(())
            }

            let message = rpc::RemoteServerRequestType::RegisterUser(requested_account, password);

            match cmd.server.server().sync_log().send_remote_request(services_name, message).await
            {
                Ok(rpc::RemoteServerResponse::LogUserIn(account)) =>
                {
                    cmd.server.add_action(CommandAction::state_change(source, event::UserLogin {
                        account: Some(account)
                    }));
                    cmd.connection.send(&message::Register::new("SUCCESS", requested_account, "You have successfully registered"));
                }
                Ok(rpc::RemoteServerResponse::AlreadyExists) =>
                {
                    cmd.connection.send(&message::Fail::new("REGISTER",
                                                            "ACCOUNT_EXISTS",
                                                            account_arg,
                                                            "Account already exists"));
                }
                Ok(response) =>
                {
                    tracing::error!(?response, "Unexpected response from services");
                    cmd.connection.send(&message::Fail::new("REGISTER",
                                                            "TEMPORARILY_UNAVAILABLE",
                                                            account_arg,
                                                            "Services are temporarily unavailable"));
                }
                Err(e) =>
                {
                    tracing::error!(?e, "Error sending register request");
                    cmd.connection.send(&message::Fail::new("REGISTER",
                                                            "TEMPORARILY_UNAVAILABLE",
                                                            account_arg,
                                                            "Services are temporarily unavailable"));
                }
            }

            Ok(())
        }))
    }
});