use messages::send_realtime::SendRealtimeItem;
use sable_network::prelude::update::{HistoricMessageSource, HistoricMessageTarget};

use super::*;
use crate::errors::HandleResult;
use crate::monitor::MonitoredItem;

impl ClientServer {
    pub(super) fn handle_history_update(&self, update: NetworkHistoryUpdate) -> HandleResult {
        tracing::trace!(?update, "Got history update");

        match update {
            NetworkHistoryUpdate::NewEntry(entry_id) => {
                let history = self.node.history();
                if let Some(entry) = history.get(entry_id) {
                    match &entry.details {
                        NetworkStateChange::NewUser(detail) => {
                            detail.notify_monitors(self);
                        }
                        NetworkStateChange::UserNickChange(detail) => {
                            detail.notify_monitors(self);
                        }
                        NetworkStateChange::UserQuit(detail) => {
                            detail.notify_monitors(self);
                        }
                        NetworkStateChange::BulkUserQuit(detail) => {
                            detail.notify_monitors(self);
                        }
                        NetworkStateChange::NewUserConnection(detail) => {
                            let new_user_connection = detail.clone();
                            drop(history);
                            self.handle_new_user_connection(&new_user_connection)?;
                        }
                        NetworkStateChange::UserConnectionDisconnected(detail) => {
                            let user_disconnect = detail.clone();
                            drop(history);
                            self.handle_user_disconnect(&user_disconnect)?;
                        }
                        NetworkStateChange::ServicesUpdate(detail) => {
                            let update = detail.clone();
                            drop(history);
                            self.handle_services_update(&update)?;
                        }
                        NetworkStateChange::EventComplete(_) => {
                            // All
                            self.stored_response_sinks
                                .write()
                                .remove(&entry.source_event);
                        }
                        _ => {}
                    }
                }
            }
            NetworkHistoryUpdate::NotifyUser(user_id, entry_id) => {
                self.notify_user_update(user_id, entry_id)?;
            }
            NetworkHistoryUpdate::NotifyUsers(user_ids, entry_id) => {
                for user_id in user_ids {
                    self.notify_user_update(user_id, entry_id)?;
                }
            }
        }

        Ok(())
    }

    fn notify_user_update(&self, user_id: UserId, entry_id: LogEntryId) -> HandleResult {
        for conn in self.connections.read().get_user(user_id) {
            let log = self.node.history();

            if let Some(entry) = log.get(entry_id) {
                let stored_sinks = self.stored_response_sinks.read();
                let sink = stored_sinks
                    .get(&entry.source_event, conn.id())
                    .unwrap_or(&conn);

                // Messages need special handling at this level because of the highly irritating interaction
                // between labeled-response and echo-message
                match &entry.details {
                    NetworkStateChange::NewMessage(msg) => {
                        self.notify_user_of_message(&conn, &sink, entry, msg)?;
                    }
                    _ => {
                        entry.send_now(&sink, entry, self)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn notify_user_of_message(
        &self,
        conn: &ClientConnection,
        sink: &dyn MessageSink,
        entry: &HistoryLogEntry,
        msg: &update::NewMessage,
    ) -> HandleResult {
        // This special handler only exists because if labeled-response and echo-message
        // are both enabled, and the source and target of the message are the same user,
        // then they need to be notified of it twice, once with the response label and once
        // without. This is called out explicitly in the labeled-response spec, and appears
        // to exist solely to make my life difficult.

        if let HistoricMessageSource::User(source) = &msg.source {
            if let HistoricMessageTarget::User(target) = &msg.target {
                // Source and target are both users. Check for self-message with the awkward caps
                if source.user.id == target.user.id {
                    // We handle this as a special case.

                    let message = message::Message::new(
                        &msg.source,
                        &msg.target,
                        msg.message.message_type,
                        &msg.message.text,
                    )
                    .with_tags_from(entry);

                    // First, send the echo-message acknowledgement, into the labeled-response sink
                    sink.send(
                        message
                            .clone()
                            .with_required_capabilities(ClientCapability::EchoMessage),
                    );
                    // Second, send the actual message delivery, into the connection directly so that we
                    // bypass labeled-response
                    conn.send(message);

                    // And we're done. Return to bypass the normal delivery
                    return Ok(());
                }
            }
        }

        entry.send_now(&sink, entry, self)
    }

    fn handle_new_user_connection(&self, detail: &update::NewUserConnection) -> HandleResult {
        let net = self.node.network();
        let user = net.user(detail.user.user.id)?;

        if let Ok(connection) = self
            .connections
            .read()
            .get_user_connection(detail.connection.id)
        {
            // `register_new_user` doesn't set the user ID on the connection; it remains a pre-client until
            // we see the registration events come back through (i.e. here)
            connection.set_user(user.id(), detail.connection.id);

            connection.send(numeric::Numeric001::new_for(
                &self.node.name().to_string(),
                &user.nick(),
                "test",
                &user.nick(),
            ));
            connection.send(numeric::Numeric002::new_for(
                &self.node.name().to_string(),
                &user.nick(),
                self.node.name(),
                self.node.version(),
            ));
            connection.send(numeric::Numeric003::new_for(
                &self.node.name().to_string(),
                &user.nick(),
                &chrono::offset::Utc::now(),
            ));
            connection.send(numeric::Numeric004::new_for(
                &self.node.name().to_string(),
                &user.nick(),
                self.node.name(),
                self.node.version(),
                &self.myinfo.user_modes,
                &self.myinfo.chan_modes,
                &self.myinfo.chan_modes_with_a_parameter,
            ));
            for line in self.isupport.data().iter() {
                connection.send(numeric::ISupport::new_for(
                    &self.node.name().to_string(),
                    &user.nick(),
                    line,
                ));
            }

            crate::utils::send_motd(self, &connection, &user)?;

            connection.send(message::Mode::new(&user, &user, &user.mode().format()));

            connection.send(message::Notice::new(&self.node.name().to_string(), &user,
                    "The network is currently running in debug mode. Do not send any sensitive information such as passwords."));
        }
        Ok(())
    }

    fn handle_user_disconnect(&self, detail: &update::UserConnectionDisconnected) -> HandleResult {
        self.connections
            .write()
            .remove_user_connection(detail.connection.id);
        Ok(())
    }

    fn handle_services_update(&self, detail: &update::ServicesUpdate) -> HandleResult {
        match &detail.new_state {
            Some(state) => {
                let mut mechanisms = state.sasl_mechanisms.clone();
                mechanisms.push("EXTERNAL".to_string());
                self.client_caps
                    .enable_with_values(ClientCapability::Sasl, &mechanisms);
            }
            None => {
                // Services has disappeared for some reason. Don't fully disable SASL, though,
                // since we can still process external auth via certificates locally
                let mechanisms = vec!["EXTERNAL".to_string()];
                self.client_caps
                    .enable_with_values(ClientCapability::Sasl, &mechanisms);
            }
        }
        Ok(())
    }
}
