use super::*;

#[command_handler("MOTD")]
fn handle_motd(server: &ClientServer, response: &dyn CommandResponse) -> CommandResult {
    match &server.infos.motd {
        None => response.numeric(make_numeric!(NoMotd)),
        Some(motd) => {
            response.numeric(make_numeric!(MotdStart, server.name()));
            for line in motd.lines() {
                response.numeric(make_numeric!(Motd, line));
            }

            response.numeric(make_numeric!(EndOfMotd));
        }
    }

    Ok(())
}
