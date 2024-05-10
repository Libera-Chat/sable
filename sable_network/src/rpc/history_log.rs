use crate::id::{EventId, UserId};
/// A message indicating that something has been added to the network history log,
/// which a history subscriber may want to do something about.
use crate::network::update::NetworkStateChange;

#[derive(Debug)]
pub struct NetworkHistoryUpdate {
    pub event: EventId,
    pub timestamp: i64,
    pub change: NetworkStateChange,
    pub users_to_notify: Vec<UserId>,
}

impl crate::history::HistoryItem for NetworkHistoryUpdate {
    fn timestamp(&self) -> i64 {
        self.timestamp
    }

    fn change(&self) -> &NetworkStateChange {
        &self.change
    }
}
