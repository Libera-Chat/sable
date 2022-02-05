
/// Describes one of the three possible actions to take when shutting down a server process.
#[derive(Debug,Clone)]
pub enum ShutdownAction
{
    Shutdown,
    Restart,
    Upgrade
}