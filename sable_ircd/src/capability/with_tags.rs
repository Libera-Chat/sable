use crate::messages::OutboundClientMessage;
use sable_network::{history::HistoryItem, network::Network};

use super::*;

pub(crate) trait WithSupportedTags {
    fn with_tags_from(self, from_update: &impl HistoryItem, net: &Network) -> Self;
}

impl WithSupportedTags for OutboundClientMessage {
    fn with_tags_from(self, from_update: &impl HistoryItem, net: &Network) -> Self {
        let server_time_tag = server_time::server_time_tag(from_update.timestamp());

        let mut result = self.with_tag(server_time_tag);

        if let Some(msgid_tag) = msgid::msgid_tag(from_update) {
            result = result.with_tag(msgid_tag);
        }
        if let Some(account_tag) = account_tag::account_tag(from_update.change(), net) {
            result = result.with_tag(account_tag);
        }

        result
    }
}
