use super::*;
use sable_network::history::*;

pub(crate) trait WithSupportedTags
{
    type Tagged;
    fn with_tags_from(self, history_entry: &HistoryLogEntry) -> Self::Tagged;
}

impl<T: TaggableMessage> WithSupportedTags for T
{
    type Tagged = <Self as TaggableMessage>::Tagged;

    fn with_tags_from(self, history_entry: &HistoryLogEntry) -> Self::Tagged
    {
        let server_time_tag = server_time::server_time_tag(history_entry.timestamp);

        self.with_tag(server_time_tag)
    }
}