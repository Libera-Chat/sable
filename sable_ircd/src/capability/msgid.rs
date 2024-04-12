use base64::prelude::*;

use super::*;
use crate::messages::OutboundMessageTag;
use sable_network::id::EventId;

pub fn msgid_tag(event_id: EventId) -> OutboundMessageTag {
    let mut buf = [0u8; 24];
    buf[0..8].copy_from_slice(&event_id.server().local().to_le_bytes());
    buf[8..16].copy_from_slice(&event_id.epoch().local().to_le_bytes());
    buf[16..24].copy_from_slice(&event_id.local().to_le_bytes());
    OutboundMessageTag::new(
        "msgid",
        Some(BASE64_URL_SAFE_NO_PAD.encode(&buf[..])),
        ClientCapability::MessageTags,
    )
}
