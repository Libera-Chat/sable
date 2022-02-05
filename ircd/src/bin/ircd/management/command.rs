use irc_server::server::ServerManagementCommand;
use rpc_protocols::ShutdownAction;

pub enum ManagementCommand
{
    ServerCommand(ServerManagementCommand),
    Shutdown(ShutdownAction),
}
