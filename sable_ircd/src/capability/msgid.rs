use super::*;
use crate::messages::OutboundMessageTag;
use sable_network::history::HistoryItem;
use sable_network::network::NetworkStateChange;

pub fn msgid_tag(from_update: &impl HistoryItem) -> Option<OutboundMessageTag> {
    match from_update.change() {
        NetworkStateChange::NewMessage(detail) => Some(OutboundMessageTag::new(
            "msgid",
            Some(detail.message.to_string()),
            ClientCapability::MessageTags,
        )),
        _ => None,
    }
}
