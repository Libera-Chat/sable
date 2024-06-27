use crate::errors::HandleResult;
use crate::messages::MessageSink;
use crate::ClientServer;
use sable_network::network::state;
use sable_network::network::state::HistoricUser;
use sable_network::prelude::wrapper::ObjectWrapper;
use sable_network::prelude::*;
use sable_network::rpc::NetworkHistoryUpdate;

use super::send_history::SendHistoryItem;
use super::*;

/// Extension trait for network updates that behave differently in realtime than in history playback
pub(crate) trait SendRealtimeItem: SendHistoryItem {
    // Default implementation delegates to the historic version
    fn send_now(
        &self,
        conn: &impl MessageSink,
        from_entry: &NetworkHistoryUpdate,
        _server: &ClientServer,
    ) -> HandleResult;
}

impl SendRealtimeItem for NetworkHistoryUpdate {
    fn send_now(
        &self,
        conn: &impl MessageSink,
        _from_entry: &NetworkHistoryUpdate,
        server: &ClientServer,
    ) -> HandleResult {
        match &self.change {
            NetworkStateChange::ChannelJoin(detail) => detail.send_now(conn, self, server),
            NetworkStateChange::ChannelRename(detail) => detail.send_now(conn, self, server),
            _ => self.send_to(conn, self),
        }
    }
}

impl SendRealtimeItem for update::ChannelJoin {
    fn send_now(
        &self,
        conn: &impl MessageSink,
        from_entry: &NetworkHistoryUpdate,
        server: &ClientServer,
    ) -> HandleResult {
        // When a user joins, we need to send topic, names, etc. We can't easily do that when replaying
        // history, because we don't have direct access to the historic member list.

        self.send_to(conn, from_entry)?;

        // If we're notifying someone other than the joining user, we're done now
        if conn.user_id() != Some(self.user.id()) {
            return Ok(());
        }

        // If we get here, the user we're notifying is the joining user
        let network = server.network();
        let channel = network.channel(self.channel.id)?;
        let user = network.user(self.user.id())?;

        if let Some(topic) = channel.topic() {
            conn.send(numeric::TopicIs::new(&channel, topic.text()).format_for(server, &self.user));
            conn.send(
                numeric::TopicSetBy::new(&channel, topic.setter(), topic.timestamp())
                    .format_for(server, &self.user),
            );
        }

        crate::utils::send_channel_names(server, conn, &user, &channel)?;

        Ok(())
    }
}

impl SendRealtimeItem for update::ChannelRename {
    fn send_now(
        &self,
        conn: &impl MessageSink,
        from_entry: &NetworkHistoryUpdate,
        server: &ClientServer,
    ) -> HandleResult {
        if conn.capabilities().has(ClientCapability::ChannelRename) {
            conn.send(
                message::Rename::new(&self.source, &self.old_name, &self.new_name, &self.message)
                    .with_tags_from(from_entry)
                    .with_required_capabilities(ClientCapability::ChannelRename),
            );

            Ok(())
        } else {
            // For clients which don't support draft/channel-rename, emulate by making
            // them PART + JOIN:

            let Some(user_id) = conn.user_id() else {
                return Ok(());
            };

            let network = server.network();
            let channel = network.channel(self.channel.id)?;
            let Some(membership) = channel.has_member(user_id) else {
                tracing::warn!("Cannot send ChannelRename to non-member {:?}", user_id);
                return Ok(());
            };

            let user = network.user(user_id)?;

            // Construct fake join/part updates so that we can fake the log entry as well

            let fake_part = update::ChannelPart {
                channel: state::Channel {
                    name: self.old_name,
                    ..self.channel.clone()
                },
                membership: membership.raw().clone(),
                user: HistoricUser::new(user.raw(), &network),
                message: format!("Channel renamed to {}: {}", &self.new_name, &self.message),
            };

            let fake_join = update::ChannelJoin {
                channel: state::Channel {
                    name: self.new_name,
                    ..self.channel.clone()
                },
                membership: membership.raw().clone(),
                user: HistoricUser::new(user.raw(), &network),
            };

            let fake_log_entry = NetworkHistoryUpdate {
                timestamp: from_entry.timestamp,
                event: from_entry.event,
                change: NetworkStateChange::ChannelJoin(fake_join),
                users_to_notify: vec![],
            };

            fake_part.send_to(conn, &fake_log_entry)?;
            // fake_join was moved into fake_log_entry
            fake_log_entry.send_now(conn, &fake_log_entry, server)
        }
    }
}
