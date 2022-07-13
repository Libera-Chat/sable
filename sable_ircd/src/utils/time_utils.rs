use chrono::prelude::*;

pub fn format_timestamp(ts: i64) -> String
{
    Utc.timestamp(ts, 0).to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn parse_timestamp(str: &str) -> Option<i64>
{
    Utc.datetime_from_str(str, "%+").map(|dt| dt.timestamp()).ok()
}