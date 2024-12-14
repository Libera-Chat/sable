use super::*;
use sable_network::prelude::wrapper::ObjectWrapper;

#[command_handler("LINKS")]
fn handle_links(
    response: &dyn CommandResponse,
    local_server: &ClientServer,
    net: &Network,
    source: UserSource,
) -> CommandResult {
    local_server.policy().can_list_links(&source)?;

    for remote_server in net.servers() {
        let remote_server_info = format!(
            "last_ping={}, version={}",
            remote_server.last_ping(),
            remote_server.raw().version
        );
        response.numeric(make_numeric!(
            Links,
            remote_server.name(),
            local_server.name(),
            0,
            &remote_server_info
        ));
    }

    response.numeric(make_numeric!(EndOfLinks));

    Ok(())
}
