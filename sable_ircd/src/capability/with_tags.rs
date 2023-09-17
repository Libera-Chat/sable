use crate::messages::OutboundClientMessage;

use super::*;
use sable_network::history::*;

pub(crate) trait WithSupportedTags {
    fn with_tags_from(self, history_entry: &HistoryLogEntry) -> Self;
}

impl WithSupportedTags for OutboundClientMessage {
    fn with_tags_from(self, history_entry: &HistoryLogEntry) -> Self {
        let server_time_tag = server_time::server_time_tag(history_entry.timestamp);

        let mut result = self.with_tag(server_time_tag);
        if let Some(account_tag) = account_tag::account_tag(&history_entry.details) {
            result = result.with_tag(account_tag);
        }

        result
    }
}
