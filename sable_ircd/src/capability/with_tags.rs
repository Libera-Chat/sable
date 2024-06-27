use crate::messages::OutboundClientMessage;
use sable_network::history::HistoryItem;

use super::*;

pub(crate) trait WithSupportedTags {
    fn with_tags_from(self, from_update: &impl HistoryItem) -> Self;
}

impl WithSupportedTags for OutboundClientMessage {
    fn with_tags_from(self, from_update: &impl HistoryItem) -> Self {
        let server_time_tag = server_time::server_time_tag(from_update.timestamp());

        let mut result = self.with_tag(server_time_tag);
        if let Some(account_tag) = account_tag::account_tag(from_update.change()) {
            result = result.with_tag(account_tag);
        }

        result
    }
}
