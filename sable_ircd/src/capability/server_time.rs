use super::*;
use super::message_tag::MessageTag;
use crate::utils::format_timestamp;

pub fn server_time_tag(ts: i64) -> MessageTag
{
    MessageTag::new("server-time", format_timestamp(ts), ClientCapability::ServerTime)
}