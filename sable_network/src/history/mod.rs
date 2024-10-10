mod log;
pub use log::*;
mod service;
pub use service::*;
mod local_service;

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
