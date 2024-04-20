use sable_network::rpc::{RemoteServerRequestType, RemoteServerResponse};

use super::*;

#[command_handler("CERT", in("NS"))]
async fn handle_cert(
    services: ServicesTarget<'_>,
    net: &Network,
    source: LoggedInUserSource<'_>,
    cmd: &dyn Command,
    subcommand: &str,
    param: Conditional<&str>,
) -> CommandResult {
    match subcommand.to_ascii_uppercase().as_str() {
        "ADD" => cert_add(services, net, source, cmd, param).await,
        "DEL" => cert_del(services, source, cmd, param).await,
        "LIST" => cert_list(source, cmd).await,
        _ => {
            cmd.notice("Invalid command. Syntax: CERT ADD|DEL|LIST [fingerprint]");
            Ok(())
        }
    }
}

async fn cert_list(source: LoggedInUserSource<'_>, cmd: &dyn Command) -> CommandResult {
    let fingerprints = source.account.fingerprints();

    if fingerprints.is_empty() {
        cmd.notice("You have no authorised certificates");
    } else {
        cmd.notice(format_args!(
            "Authorised certificates for account {}:",
            source.account.name()
        ));

        for fp in fingerprints.iter() {
            cmd.notice(format_args!(" - {}", fp));
        }
    }
    Ok(())
}

async fn cert_add(
    services: ServicesTarget<'_>,
    net: &Network,
    source: LoggedInUserSource<'_>,
    cmd: &dyn Command,
    param: Conditional<&str>,
) -> CommandResult {
    let fingerprint = match param.is_present() {
        Some(fp) => fp,
        None => {
            if let Some(fp) = cmd
                .connection()
                .tls_info()
                .and_then(|ti| ti.fingerprint.as_deref())
            {
                fp
            } else {
                cmd.notice(
                    "You must specify a certificate fingerprint if you're not currently using one",
                );
                return Ok(());
            }
        }
    };

    if net.account_with_fingerprint(fingerprint).is_some() {
        cmd.notice("That fingerprint is already in use");
        return Ok(());
    }

    let req =
        RemoteServerRequestType::AddAccountFingerprint(source.account.id(), fingerprint.to_owned());

    match services.send_remote_request(req).await {
        Ok(RemoteServerResponse::Success) => {
            cmd.notice(format_args!(
                "Fingerprint {} has been added to your account",
                fingerprint
            ));
        }
        Ok(response) => {
            tracing::warn!(?response, "Unexpected response to fingerprint add message");
            cmd.notice("Error adding fingerprint");
        }
        Err(e) => {
            tracing::warn!(?e, "Error response adding fingerprint");
            cmd.notice("Error adding fingerprint");
        }
    }

    Ok(())
}

async fn cert_del(
    services: ServicesTarget<'_>,
    source: LoggedInUserSource<'_>,
    cmd: &dyn Command,
    param: Conditional<&str>,
) -> CommandResult {
    let fingerprint = param.require()?;

    let req = RemoteServerRequestType::RemoveAccountFingerprint(
        source.account.id(),
        fingerprint.to_owned(),
    );

    match services.send_remote_request(req).await {
        Ok(RemoteServerResponse::Success) => {
            cmd.notice(format_args!(
                "Fingerprint {} has been removed from your account",
                fingerprint
            ));
        }
        Ok(response) => {
            tracing::warn!(
                ?response,
                "Unexpected response to fingerprint remove message"
            );
            cmd.notice("Error removing fingerprint");
        }
        Err(e) => {
            tracing::warn!(?e, "Error response removing fingerprint");
            cmd.notice("Error removing fingerprint");
        }
    }

    Ok(())
}
