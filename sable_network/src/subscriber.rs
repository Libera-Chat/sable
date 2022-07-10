use crate::prelude::*;

pub use tokio::sync::mpsc::{
    Sender,
};

pub trait HistorySubscriber
{
    /// Called to notify that a new history log entry has been created. At this point no
    /// users should be notified of it; `notify_user` will be called for each user ID that
    /// should be able to view this event.
    fn new_log_entry(&self, entry: &HistoryLogEntry);

    /// Called to notify the subscriber that a given user ID should be permitted to see the
    /// given log entry details.
    fn notify_user(&self, user_id: UserId, entry: &HistoryLogEntry);
}

pub struct ChannelHistorySubscriber
{

}