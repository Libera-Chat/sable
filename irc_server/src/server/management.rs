use super::*;

use tokio::sync::oneshot::Sender;

/// A management command
pub enum ServerManagementCommand
{
    /// Collect server statistics
    ServerStatistics(Sender<String>)
}

/// Statistics to be exported via the management interface
#[derive(serde::Serialize)]
struct ServerStatistics
{
    connected_clients: usize
}

impl Server
{
    pub(super) async fn handle_management_command(&mut self, cmd: ServerManagementCommand)
    {
        use ServerManagementCommand::*;
        match cmd
        {
            ServerStatistics(chan) =>
            {
                let _ = chan.send(self.export_server_statistics().await);
            }
        }
    }

    async fn export_server_statistics(&self) -> String
    {
        let stats = ServerStatistics {
            connected_clients: self.connections.len()
        };

        serde_json::to_string(&stats).expect("Failed to serialise statistics")
    }
}