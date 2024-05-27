use super::*;

use itertools::Itertools;

#[command_handler("HELP", "UHELP")]
/// HELP [<topic>]
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
    // TODO: oper help? (and if oper help is on the same command, UHELP like solanum?)
    // TODO: non-command help topics
    let is_oper = command.command().to_ascii_uppercase() != "UHELP" && source.is_oper();

    match topic {
        Some(s) => {
            let topic = s.to_ascii_uppercase();
            let topic = topic
                .split_once(' ')
                .map_or(topic.clone(), |(t, _)| t.to_string());

            if let Some(cmd) = server.get_command(&topic) {
                if cmd.docs.len() > 0 {
                    // TODO
                    if cmd.restricted && is_oper {
                        response.numeric(make_numeric!(HelpNotFound, &topic));
                        return Ok(());
                    }
                    let mut lines = cmd.docs.iter();
                    response.numeric(make_numeric!(
                        HelpStart,
                        &topic,
                        lines.next().unwrap_or(&topic.as_str())
                    ));
                    for line in lines {
                        response.numeric(make_numeric!(HelpText, &topic, line));
                    }
                    response.numeric(make_numeric!(EndOfHelp, &topic));
                    return Ok(());
                }
            }
            response.numeric(make_numeric!(HelpNotFound, &topic));
        }
        None => {
            let topic = "*";
            response.numeric(make_numeric!(HelpStart, topic, "Available help topics:"));
            response.numeric(make_numeric!(HelpText, topic, ""));
            for chunk in &server
                .iter_commands()
                .filter_map(|(k, v)| {
                    if !v.restricted || is_oper {
                        Some(k.to_ascii_uppercase())
                    } else {
                        None
                    }
                })
                .sorted()
                .chunks(4)
            {
                let line = format!("{:16}", chunk.format(" "));
                response.numeric(make_numeric!(HelpText, topic, &line));
            }
            response.numeric(make_numeric!(EndOfHelp, topic));
        }
    };
    Ok(())
}
