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