use super::*;
use crate::messages::OutboundMessageTag;
use crate::utils::format_timestamp;

pub fn server_time_tag(ts: i64) -> OutboundMessageTag {
    OutboundMessageTag::new(
        "server-time",
        Some(format_timestamp(ts)),
        ClientCapability::ServerTime,
    )
}
