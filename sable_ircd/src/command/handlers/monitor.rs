//! Implementation of the UI of [IRCv3 MONITOR](https://ircv3.net/specs/extensions/monitor)

use super::*;
use crate::monitor::MonitorInsertError;
use crate::utils::LineWrapper;

const MAX_CONTENT_LENGTH: usize = 300; // Conservative limit to avoid hitting 512 bytes limit

#[command_handler("MONITOR")]
fn handle_monitor(
    server: &ClientServer,
    cmd: &dyn Command,
    subcommand: &str,
    targets: Option<&str>,
) -> CommandResult {
    match subcommand.to_ascii_uppercase().as_str() {
        "+" => handle_monitor_add(server, cmd, targets),
        "-" => handle_monitor_del(server, cmd, targets),
        "C" => handle_monitor_clear(server, cmd),
        "L" => handle_monitor_list(server, cmd),
        "S" => handle_monitor_show(server, cmd),
        _ => Ok(()), // The spec does not say what to do; existing implementations ignore it
    }
}

fn handle_monitor_add(
    server: &ClientServer,
    cmd: &dyn Command,
    targets: Option<&str>,
) -> CommandResult {
    let targets = targets
        .ok_or(CommandError::NotEnoughParameters)? // technically we could just ignore
        .split(',')
        .map(|target| Nickname::parse_str(cmd, target))
        .collect::<Result<Vec<_>, _>>()?; // ditto
    let mut monitors = server.monitors.write();
    let res = targets
        .iter()
        .try_for_each(|&target| monitors.insert(target, cmd.connection_id()))
        .map_err(
            |MonitorInsertError::TooManyMonitorsPerConnection { max, current }| {
                CommandError::Numeric(make_numeric!(MonListFull, max, current))
            },
        );
    drop(monitors); // Release lock
    send_statuses(cmd, targets);
    res
}

fn handle_monitor_del(
    server: &ClientServer,
    cmd: &dyn Command,
    targets: Option<&str>,
) -> CommandResult {
    let targets = targets
        .ok_or(CommandError::NotEnoughParameters)? // technically we could just ignore
        .split(',')
        .map(|target| Nickname::parse_str(cmd, target))
        .collect::<Result<Vec<_>, _>>()?; // ditto

    let mut monitors = server.monitors.write();
    for target in targets {
        monitors.remove(target, cmd.connection_id());
    }
    Ok(())
}

fn handle_monitor_clear(server: &ClientServer, cmd: &dyn Command) -> CommandResult {
    server
        .monitors
        .write()
        .remove_connection(cmd.connection_id());
    Ok(())
}

fn handle_monitor_list(server: &ClientServer, cmd: &dyn Command) -> CommandResult {
    // Copying the set of monitors to release lock on `server.monitors` ASAP
    let monitors: Option<Vec<_>> = server
        .monitors
        .read()
        .monitored_nicks(cmd.connection_id())
        .map(|monitors| monitors.iter().copied().collect());

    if let Some(monitors) = monitors {
        LineWrapper::<',', _, _>::new(MAX_CONTENT_LENGTH, monitors.into_iter())
            .for_each(|line| cmd.numeric(make_numeric!(MonList, &line)));
    }
    cmd.numeric(make_numeric!(EndOfMonList));

    Ok(())
}

fn handle_monitor_show(server: &ClientServer, cmd: &dyn Command) -> CommandResult {
    // Copying the set of monitors to release lock on `server.monitors` ASAP
    let monitors: Option<Vec<_>> = server
        .monitors
        .read()
        .monitored_nicks(cmd.connection_id())
        .map(|monitors| monitors.iter().copied().collect());

    if let Some(monitors) = monitors {
        send_statuses(cmd, monitors);
    }
    Ok(())
}

fn send_statuses(cmd: &dyn Command, targets: Vec<Nickname>) {
    let mut online = Vec::new();
    let mut offline = Vec::new();
    for target in targets {
        match cmd.network().user_by_nick(&target) {
            Ok(user) => online.push(user.nuh()),
            Err(LookupError::NoSuchNick(_)) => offline.push(target),
            Err(e) => {
                tracing::error!(
                    "Unexpected error while computing online status of {}: {}",
                    target,
                    e
                );
            }
        }
    }

    LineWrapper::<',', _, _>::new(MAX_CONTENT_LENGTH, online.into_iter())
        .for_each(|line| cmd.numeric(make_numeric!(MonOnline, &line)));
    LineWrapper::<',', _, _>::new(MAX_CONTENT_LENGTH, offline.into_iter())
        .for_each(|line| cmd.numeric(make_numeric!(MonOffline, &line)));
}
