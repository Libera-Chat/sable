use irc_server::server::ServerManagementCommand;

pub enum ManagementCommand
{
    ServerCommand(ServerManagementCommand)
}