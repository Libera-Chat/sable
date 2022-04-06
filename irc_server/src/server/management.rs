use super::*;

use tokio::sync::oneshot::Sender;

/// A management command
pub struct ServerManagementCommand
{
    pub cmd: ServerManagementCommandType,
    pub response: Sender<String>
}

pub enum ServerManagementCommandType
{
    /// Collect server statistics
    ServerStatistics,
    /// Dump network state (for debugging)
    DumpNetwork,
    /// Dump event log (for debugging)
    DumpEvents,
}

/// Statistics to be exported via the management interface
#[derive(serde::Serialize)]
struct ServerStatistics
{
    connected_clients: usize,
    event_stats: ircd_sync::EventLogStats,
}

impl Server
{
    pub(super) async fn handle_management_command(&mut self, cmd: ServerManagementCommand)
    {
        use ServerManagementCommandType::*;
        let resp = match cmd.cmd
        {
            ServerStatistics => self.export_server_statistics(),
            DumpNetwork => self.dump_network_state(),
            DumpEvents => self.dump_events(),
        };
        let _ = cmd.response.send(resp);
    }

    fn export_server_statistics(&self) -> String
    {
        let stats = ServerStatistics {
            connected_clients: self.connections.len(),
            event_stats: self.event_log.event_log().get_stats(),
        };

        serde_json::to_string(&stats).expect("Failed to serialise statistics")
    }
}

#[cfg(feature="debug")]
impl Server
{
    fn dump_network_state(&self) -> String
    {
        serde_json::to_string(&self.net).expect("Failed to serialise network")
    }

    fn dump_events(&self) -> String
    {
        let log = self.event_log.event_log();
        let events = log.all_events().collect::<Vec<_>>();
        serde_json::to_string(&events).expect("Failed to serialise events")
    }
}

#[cfg(not(feature="debug"))]
impl Server
{
    fn dump_network_state(&self) -> String
    {
        "{\"error\": \"Debug functionality not enabled\"}".to_string()
    }
    fn dump_events(&self) -> String
    {
        "{\"error\": \"Debug functionality not enabled\"}".to_string()
    }
}