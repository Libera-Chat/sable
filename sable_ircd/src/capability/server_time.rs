use super::*;
use chrono::prelude::*;
use super::message_tag::MessageTag;

fn format_time_for_tag(ts: i64) -> String
{
    Utc.timestamp(ts, 0).to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn server_time_tag(ts: i64) -> MessageTag
{
    MessageTag::new("server-time", format_time_for_tag(ts), ClientCapability::ServerTime)
}