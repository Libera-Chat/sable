/// A message indicating that something has been added to the network history log,
/// which a history subscriber may want to do something about.

use crate::history::LogEntryId;
use crate::id::UserId;

#[derive(Debug)]
pub enum NetworkHistoryUpdate
{
    NewEntry(LogEntryId),
    NotifyUser(UserId, LogEntryId),
    NotifyUsers(Vec<UserId>, LogEntryId),
}