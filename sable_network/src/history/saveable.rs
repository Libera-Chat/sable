use super::*;
use crate::prelude::*;
use serde::{Serialize,Deserialize};
use std::collections::HashMap;

/// Saved state of a [`NetworkHistoryLog`]
#[derive(Debug,Serialize,Deserialize)]
pub struct NetworkHistoryLogState
{
    entries: HashMap<LogEntryId, HistoryLogEntry>,
    last_entry_id: LogEntryId,
    user_logs: HashMap<UserId, Vec<LogEntryId>>,
}

impl crate::saveable::Saveable for NetworkHistoryLog
{

}