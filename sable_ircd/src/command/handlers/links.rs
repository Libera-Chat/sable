use super::*;
use sable_network::prelude::wrapper::ObjectWrapper;

#[command_handler("LINKS")]
fn handle_links(
    response: &dyn CommandResponse,
    server: &ClientServer,
    net: &Network,
    source: UserSource,
) -> CommandResult {
    server.policy().can_list_links(&source)?;

    for server in net.servers() {
        let server_info = format!(
            "last_ping={}, version={}",
            server.last_ping(),
            server.raw().version
        );
        response.numeric(make_numeric!(Links, server.name(), 0, &server_info));
    }

    response.numeric(make_numeric!(EndOfLinks));

    Ok(())
}
