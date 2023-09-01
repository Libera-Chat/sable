use super::*;

#[command_handler("MOTD")]
fn handle_motd(server: &ClientServer, response: &dyn CommandResponse) -> CommandResult {
    match &server.info_strings.motd {
        None => response.numeric(make_numeric!(NoMotd)),
        Some(motd) => {
            response.numeric(make_numeric!(MotdStart, server.name()));
            for ele in motd {
                response.numeric(make_numeric!(Motd, ele))
            }

            response.numeric(make_numeric!(EndOfMotd));
        }
    }

    Ok(())
}
