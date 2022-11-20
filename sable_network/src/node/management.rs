use super::*;

/// Statistics to be exported via the management interface
#[derive(serde::Serialize)]
struct ServerStatistics
{
    connected_clients: usize,
    event_stats: crate::sync::EventLogStats,
}

impl NetworkNode
{
    pub async fn handle_management_command(&self, cmd: ServerManagementCommand)
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
        unimplemented!("server stats");
/*
        let stats = ServerStatistics {
            connected_clients: self.connections.len(),
            event_stats: self.server.event_log().get_stats(),
        };

        serde_json::to_string(&stats).expect("Failed to serialise statistics")
*/
    }
}

#[cfg(feature="debug")]
impl NetworkNode
{
    fn dump_network_state(&self) -> String
    {
        serde_json::to_string(&*self.network()).expect("Failed to serialise network")
    }

    fn dump_events(&self) -> String
    {
        let log = self.event_log();
        let events = log.all_events().collect::<Vec<_>>();
        serde_json::to_string(&events).expect("Failed to serialise events")
    }
}

#[cfg(not(feature="debug"))]
impl NetworkNode
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