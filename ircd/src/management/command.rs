use sable_ircd::server::ServerManagementCommand;
use sable_network::rpc::ShutdownAction;

pub enum ManagementCommand
{
    ServerCommand(ServerManagementCommand),
    Shutdown(ShutdownAction),
}
