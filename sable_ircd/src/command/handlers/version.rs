use super::*;

#[command_handler("VERSION")]
fn handle_version(server: &ClientServer, response: &dyn CommandResponse) -> CommandResult {
    response.numeric(make_numeric!(
        Version,
        server.name(),
        server.node().version()
    ));

    for v in server.isupport.data().iter() {
        response.numeric(make_numeric!(ISupport, v))
    }

    Ok(())
}
