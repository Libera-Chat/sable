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
pub(crate) trait SendRealtimeItem<Item>: SendHistoryItem<Item> {
    // Default implementation delegates to the historic version
    fn send_now(
        &self,
        item: &Item,
        conn: &impl MessageSink,
        from_entry: &NetworkHistoryUpdate,
    ) -> HandleResult;
}

impl SendRealtimeItem<NetworkHistoryUpdate> for ClientServer {
    fn send_now(
        &self,
        item: &NetworkHistoryUpdate,
        conn: &impl MessageSink,
        _from_entry: &NetworkHistoryUpdate,
    ) -> HandleResult {
        match &item.change {
            NetworkStateChange::ChannelJoin(detail) => self.send_now(detail, conn, item),
            NetworkStateChange::ChannelRename(detail) => self.send_now(detail, conn, item),
            _ => self.send_item(item, conn, item),
        }
    }
}

impl SendRealtimeItem<update::ChannelJoin> for ClientServer {
    fn send_now(
        &self,
        item: &update::ChannelJoin,
        conn: &impl MessageSink,
        from_entry: &NetworkHistoryUpdate,
    ) -> HandleResult {
        // When a user joins, we need to send topic, names, etc. We can't easily do that when replaying
        // history, because we don't have direct access to the historic member list.

        self.send_item(item, conn, from_entry)?;

        // If we're notifying someone other than the joining user, we're done now
        if conn.user_id() != Some(*item.user.user()) {
            return Ok(());
        }

        // If we get here, the user we're notifying is the joining user
        let network = self.network();
        let channel = network.channel(item.channel.id)?;
        let user = network.user(*item.user.user())?;

        if let Some(topic) = channel.topic() {
            conn.send(numeric::TopicIs::new(&channel, topic.text()).format_for(self, &user));
            conn.send(
                numeric::TopicSetBy::new(&channel, topic.setter(), topic.timestamp())
                    .format_for(self, &user),
            );
        }

        crate::utils::send_channel_names(self, conn, &user, &channel)?;

        Ok(())
    }
}

impl SendRealtimeItem<update::ChannelRename> for ClientServer {
    fn send_now(
        &self,
        item: &update::ChannelRename,
        conn: &impl MessageSink,
        from_entry: &NetworkHistoryUpdate,
    ) -> HandleResult {
        let net = self.network();
        let source = net.message_source(&item.source)?;

        if conn.capabilities().has(ClientCapability::ChannelRename) {
            conn.send(
                message::Rename::new(&source, &item.old_name, &item.new_name, &item.message)
                    .with_tags_from(from_entry, &net)
                    .with_required_capabilities(ClientCapability::ChannelRename),
            );

            Ok(())
        } else {
            // For clients which don't support draft/channel-rename, emulate by making
            // them PART + JOIN:

            let Some(user_id) = conn.user_id() else {
                return Ok(());
            };

            let network = self.network();
            let channel = network.channel(item.channel.id)?;
            let Some(membership) = channel.has_member(user_id) else {
                tracing::warn!("Cannot send ChannelRename to non-member {:?}", user_id);
                return Ok(());
            };

            let user = network.user(user_id)?;

            // Construct fake join/part updates so that we can fake the log entry as well

            let fake_part = update::ChannelPart {
                channel: state::Channel {
                    name: item.old_name,
                    ..item.channel.clone()
                },
                membership: membership.raw().clone(),
                user: user.historic_id(),
                message: format!("Channel renamed to {}: {}", &item.new_name, &item.message),
            };

            let fake_join = update::ChannelJoin {
                channel: state::Channel {
                    name: item.new_name,
                    ..item.channel.clone()
                },
                membership: membership.raw().clone(),
                user: user.historic_id(),
            };

            let fake_log_entry = NetworkHistoryUpdate {
                timestamp: from_entry.timestamp,
                event: from_entry.event,
                change: NetworkStateChange::ChannelJoin(fake_join),
                users_to_notify: vec![],
            };

            self.send_item(&fake_part, conn, &fake_log_entry)?;
            // fake_join was moved into fake_log_entry
            self.send_now(&fake_log_entry, conn, &fake_log_entry)
        }
    }
}
