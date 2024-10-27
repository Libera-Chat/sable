mod log;
pub use log::*;
mod service;
pub use service::*;
mod local_service;
pub use local_service::LocalHistoryService;
mod remote_service;
pub use remote_service::RemoteHistoryService;
mod tiered_service;
pub use tiered_service::TieredHistoryService;

use crate::network::NetworkStateChange;

/// Implemented by types that provide metadata for a historic state change
pub trait HistoryItem {
    fn timestamp(&self) -> i64;
    fn change(&self) -> &NetworkStateChange;
}

impl HistoryItem for HistoryLogEntry {
    fn timestamp(&self) -> i64 {
        self.timestamp
    }

    fn change(&self) -> &NetworkStateChange {
        &self.details
    }
}

/// A more concrete representation of [`HistoryItem`], with all its fields inflated
/// to strings that will be sent to the client
pub enum HistoryMessage {}
