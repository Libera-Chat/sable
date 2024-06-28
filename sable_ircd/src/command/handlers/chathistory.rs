use sable_history::{HistoryError, HistoryRequest, HistoryService};

use super::*;
use crate::{capability::ClientCapability, utils};
use messages::send_history::SendHistoryItem;

use std::cmp::{max, min};

fn parse_msgref(subcommand: &str, target: Option<&str>, msgref: &str) -> Result<i64, CommandError> {
    match msgref.split_once('=') {
        Some(("timestamp", ts)) => utils::parse_timestamp(ts).ok_or_else(|| CommandError::Fail {
            command: "CHATHISTORY",
            code: "INVALID_PARAMS",
            context: subcommand.to_string(),
            description: "Invalid timestamp".to_string(),
        }),
        Some(("msgid", _)) => Err(CommandError::Fail {
            command: "CHATHISTORY",
            code: "INVALID_MSGREFTYPE",
            context: match target {
                Some(target) => format!("{} {}", subcommand, target),
                None => subcommand.to_string(),
            },
            description: "msgid-based history requests are not supported yet".to_string(),
        }),
        _ => Err(CommandError::Fail {
            command: "CHATHISTORY",
            code: "INVALID_MSGREFTYPE",
            context: match target {
                Some(target) => format!("{} {}", subcommand, target),
                None => subcommand.to_string(),
            },
            description: format!("{:?} is not a valid message reference", msgref),
        }),
    }
}

fn parse_limit(s: &str) -> Result<usize, CommandError> {
    s.parse().map_err(|_| CommandError::Fail {
        command: "CHATHISTORY",
        code: "INVALID_PARAMS",
        context: "".to_string(),
        description: "Invalid limit".to_string(),
    })
}

#[allow(clippy::too_many_arguments)]
#[command_handler("CHATHISTORY")]
fn handle_chathistory(
    ctx: &dyn Command,
    source: UserSource,
    server: &ClientServer,
    response: &dyn CommandResponse,
    subcommand: &str,
    arg_1: &str,
    arg_2: &str,
    arg_3: &str,
    arg_4: Option<&str>,
) -> CommandResult {
    let source = source.deref();

    match subcommand.to_ascii_uppercase().as_str() {
        "TARGET" => {
            let from_ts = parse_msgref(subcommand, None, arg_1)?;
            let to_ts = parse_msgref(subcommand, None, arg_2)?;
            let limit = parse_limit(arg_3)?;

            // The spec allows the from and to timestamps in either order; list_targets requires from < to
            list_targets(
                server,
                response,
                source,
                Some(min(from_ts, to_ts)),
                Some(max(from_ts, to_ts)),
                Some(limit),
            );
        }
        normalized_subcommand => {
            let target = arg_1;
            let invalid_target_error = || CommandError::Fail {
                command: "CHATHISTORY",
                code: "INVALID_TARGET",
                context: format!("{} {}", subcommand, target),
                description: format!("Cannot fetch history from {}", target),
            };
            let target_id = TargetParameter::parse_str(ctx, target)
                .map_err(|_| invalid_target_error())?
                .into();
            let request = match normalized_subcommand {
                "LATEST" => {
                    let to_ts = match arg_2 {
                        "*" => None,
                        _ => Some(parse_msgref(subcommand, Some(target), arg_2)?),
                    };
                    let limit = parse_limit(arg_3)?;

                    HistoryRequest::Latest { to_ts, limit }
                }
                "BEFORE" => {
                    let from_ts = parse_msgref(subcommand, Some(target), arg_2)?;
                    let limit = parse_limit(arg_3)?;

                    HistoryRequest::Before { from_ts, limit }
                }
                "AFTER" => {
                    let start_ts = parse_msgref(subcommand, Some(target), arg_2)?;
                    let limit = parse_limit(arg_3)?;

                    HistoryRequest::After { start_ts, limit }
                }
                "AROUND" => {
                    let around_ts = parse_msgref(subcommand, Some(target), arg_2)?;
                    let limit = parse_limit(arg_3)?;

                    HistoryRequest::Around { around_ts, limit }
                }
                "BETWEEN" => {
                    let start_ts = parse_msgref(subcommand, Some(target), arg_2)?;
                    let end_ts = parse_msgref(subcommand, Some(target), arg_3)?;
                    let limit = parse_limit(arg_4.unwrap_or(""))?;

                    HistoryRequest::Between {
                        start_ts,
                        end_ts,
                        limit,
                    }
                }
                _ => {
                    response.send(message::Fail::new(
                        "CHATHISTORY",
                        "INVALID_PARAMS",
                        subcommand,
                        "Invalid subcommand",
                    ));
                    return Ok(());
                }
            };

            let log = server.node().history();
            match log.get_entries(source.id(), target_id, request) {
                Ok(entries) => send_history_entries(server, response, target, entries)?,
                Err(HistoryError::InvalidTarget(_)) => Err(invalid_target_error())?,
            };
        }
    }

    Ok(())
}

// For listing targets, we iterate backwards through time; this allows us to just collect the
// first timestamp we see for each target and know that it's the most recent one
fn list_targets(
    server: &ClientServer,
    into: impl MessageSink,
    source: &wrapper::User,
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    limit: Option<usize>,
) {
    let log = server.node().history();

    let found_targets = log.list_targets(source.id(), to_ts, from_ts, limit);

    // The appropriate cap here is Batch - chathistory is enabled because we got here,
    // but can be used without batch support.
    let batch = into
        .batch("chathistory-targets", ClientCapability::Batch)
        .start();

    for (target, timestamp) in found_targets {
        let target = match target {
            sable_history::TargetId::User(user) => server
                .node()
                .network()
                .user(user)
                .expect("History service returned unknown user id")
                .nick()
                .format(),
            sable_history::TargetId::Channel(channel) => server
                .node()
                .network()
                .channel(channel)
                .expect("History service returned unknown channel id")
                .name()
                .to_string(),
        };
        batch.send(message::ChatHistoryTarget::new(
            &target,
            &utils::format_timestamp(timestamp),
        ))
    }
}

fn send_history_entries<'a>(
    server: &ClientServer,
    into: impl MessageSink,
    target: &str,
    entries: impl Iterator<Item = &'a HistoryLogEntry>,
) -> CommandResult {
    let batch = into
        .batch("chathistory", ClientCapability::Batch)
        .with_arguments(&[target])
        .start();

    for entry in entries {
        server.send_item(entry, &batch, entry)?;
    }

    Ok(())
}
