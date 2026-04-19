use super::*;

use itertools::Itertools;

#[command_handler("HELP", "UHELP")]
/// HELP \[\<topic\>\]
///
/// HELP displays information for topic requested.
/// If no topic is requested, it will list available
/// help topics.
fn help_handler(
    command: &dyn Command,
    response: &dyn CommandResponse,
    server: &ClientServer,
    source: UserSource,
    topic: Option<&str>,
) -> CommandResult {
    // TODO: better restricted mechanism?
    // TODO: non-command help topics
    let is_oper = command.command().to_ascii_uppercase() != "UHELP" && source.is_oper();

    match topic {
        Some(t) => {
            let topic = t.to_ascii_uppercase();
            let topic = topic.split_once(' ').map_or(topic.clone(), |(t, _)| t.to_string());
            if let Some(mut lines) = get_help(&server.command_dispatcher, &topic, is_oper) {
                response.numeric(make_numeric!(
                    HelpStart,
                    &topic,
                    lines.next().unwrap().as_ref()
                ));
                for line in lines {
                    response.numeric(make_numeric!(HelpText, &topic, line.as_ref()));
                }
                response.numeric(make_numeric!(EndOfHelp, &topic));
                return Ok(());
            } else {
                response.numeric(make_numeric!(HelpNotFound, &topic));
            }
        }
        None => {
            let topic = "*";
            response.numeric(make_numeric!(HelpStart, &topic, "Available help topics:"));
            response.numeric(make_numeric!(HelpText, &topic, ""));
            for line in list_help(&server.command_dispatcher, is_oper) {
                response.numeric(make_numeric!(HelpText, &topic, line.as_ref()));
            }
            response.numeric(make_numeric!(EndOfHelp, &topic));
        }
    };
    Ok(())
}

#[command_handler("HELP", "UHELP", in("NS"))]
/// NS HELP \[\<topic\>\]
///
/// Displays information about the topic requested.
fn ns_help_handler(
    command: &dyn Command,
    response: &dyn CommandResponse,
    server: &ClientServer,
    source: UserSource,
    topic: Option<&str>,
) -> CommandResult {
    let is_oper = command.command().to_ascii_uppercase() != "UHELP" && source.is_oper();
    let dispatcher = CommandDispatcher::with_category("NS");

    match topic {
        Some(t) => {
            let topic = t.to_ascii_uppercase();
            let topic = topic.split_once(' ').map_or(topic.clone(), |(t, _)| t.to_string());
            if let Some(mut lines) = get_help(&dispatcher, &topic, is_oper) {
                response.numeric(make_numeric!(
                    HelpStart,
                    &topic,
                    lines.next().unwrap().as_ref()
                ));
                for line in lines {
                    response.numeric(make_numeric!(HelpText, &topic, line.as_ref()));
                }
                response.numeric(make_numeric!(EndOfHelp, &topic));
                return Ok(());
            } else {
                response.numeric(make_numeric!(HelpNotFound, &topic));
            }
        }
        None => {
            let topic = "*";
            response.numeric(make_numeric!(HelpStart, &topic, "Available help topics:"));
            response.numeric(make_numeric!(HelpText, &topic, ""));
            for line in list_help(&dispatcher, is_oper) {
                response.numeric(make_numeric!(HelpText, &topic, line.as_ref()));
            }
            response.numeric(make_numeric!(EndOfHelp, &topic));
        }
    };
    Ok(())
}

fn get_help(
    dispatcher: &CommandDispatcher,
    topic: &str,
    is_oper: bool,
) -> Option<impl Iterator<Item = impl AsRef<str>>> {
    if let Some(cmd) = dispatcher.get_command(&topic) {
        if cmd.docs.len() > 0 {
            if cmd.restricted && !is_oper {
                return None;
            }
            return Some(cmd.docs.iter());
        }
    }
    return None;
}

fn list_help(dispatcher: &CommandDispatcher, is_oper: bool) -> Vec<impl AsRef<str>> {
    let mut lines = vec![];
    for chunk in &dispatcher
        .iter_commands()
        .filter_map(|(k, v)| {
            if (!v.restricted || is_oper) && (v.docs.len() > 0) {
                Some(k.to_ascii_uppercase())
            } else {
                None
            }
        })
        .sorted()
        .chunks(4)
    {
        lines.push(format!("{:16}", chunk.format(" ")));
    }
    lines
}
