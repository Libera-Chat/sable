use sable_network::prelude::*;
use crate::ClientServer;
use crate::messages::MessageSink;
use crate::errors::HandleResult;

use super::*;
use super::send_history::SendHistoryItem;

/// Extension trait for network updates that behave differently in realtime than in history playback
pub(crate) trait SendRealtimeItem : SendHistoryItem
{
    // Default implementation delegates to the historic version
    fn send_now(&self, conn: &impl MessageSink, from_entry: &HistoryLogEntry, _server: &ClientServer) -> HandleResult;
}

impl SendRealtimeItem for HistoryLogEntry
{
    fn send_now(&self, conn: &impl MessageSink, _from_entry: &HistoryLogEntry, server: &ClientServer) -> HandleResult
    {
        match &self.details
        {
            NetworkStateChange::ChannelJoin(detail) => detail.send_now(conn, self, server),
            _ => self.send_to(conn, self)
        }
    }
}

impl SendRealtimeItem for update::ChannelJoin
{
    fn send_now(&self, conn: &impl MessageSink, from_entry: &HistoryLogEntry, server: &ClientServer) -> HandleResult
    {
        // When a user joins, we need to send topic, names, etc. We can't easily do that when replaying
        // history, because we don't have direct access to the historic member list.

        self.send_to(conn, from_entry)?;

        // If we're notifying someone other than the joining user, we're done now
        if conn.user_id() != Some(self.user.user.id)
        {
            return Ok(());
        }

        // If we get here, the user we're notifying is the joining user
        let network = server.network();
        let channel = network.channel(self.channel.id)?;
        let user = network.user(self.user.user.id)?;

        if let Some(topic) = channel.topic()
        {
            conn.send(numeric::TopicIs::new(&channel, topic.text())
                          .format_for(server, &self.user));
            conn.send(numeric::TopicSetBy::new(&channel, topic.setter(), topic.timestamp())
                          .format_for(server, &self.user));
        }

        crate::utils::send_channel_names(server, conn, &user, &channel)?;

        Ok(())
    }
}
