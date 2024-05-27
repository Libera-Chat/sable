use messages::send_realtime::SendRealtimeItem;
use sable_network::prelude::state::{HistoricMessageSourceId, HistoricMessageTargetId};

use super::*;
use crate::errors::HandleResult;
use crate::monitor::MonitoredItem;

impl ClientServer {
    pub(super) fn handle_history_update(&self, update: NetworkHistoryUpdate) -> HandleResult {
        tracing::trace!(?update, "Got history update");

        match &update.change {
            NetworkStateChange::NewUser(detail) => {
                detail.notify_monitors(self);
            }
            NetworkStateChange::UserNickChange(detail) => {
                detail.notify_monitors(self);
            }
            NetworkStateChange::UserQuit(detail) => {
                detail.notify_monitors(self);
            }
            NetworkStateChange::NewUserConnection(detail) => {
                let new_user_connection = detail.clone();
                self.handle_new_user_connection(&new_user_connection)?;
            }
            NetworkStateChange::UserConnectionDisconnected(detail) => {
                let user_disconnect = detail.clone();
                self.handle_user_disconnect(&user_disconnect)?;
            }
            NetworkStateChange::ServicesUpdate(detail) => {
                let update = detail.clone();
                self.handle_services_update(&update)?;
            }
            NetworkStateChange::EventComplete(_) => {
                // All
                self.stored_response_sinks.write().remove(&update.event);
            }
            _ => {}
        }
        for user_id in &update.users_to_notify {
            self.notify_user_update(user_id, &update)?;
        }

        Ok(())
    }

    fn notify_user_update(&self, user_id: &UserId, update: &NetworkHistoryUpdate) -> HandleResult {
        for conn in self.connections.read().get_user(*user_id) {
            let stored_sinks = self.stored_response_sinks.read();
            let sink = stored_sinks.get(&update.event, conn.id()).unwrap_or(&conn);

            // Messages need special handling at this level because of the highly irritating interaction
            // between labeled-response and echo-message
            match &update.change {
                NetworkStateChange::NewMessage(msg) => {
                    self.notify_user_of_message(&conn, &sink, update, msg)?;
                }
                _ => {
                    self.send_now(update, &sink, update)?;
                }
            }
        }

        Ok(())
    }

    fn notify_user_of_message(
        &self,
        conn: &ClientConnection,
        sink: &dyn MessageSink,
        update: &NetworkHistoryUpdate,
        msg: &update::NewMessage,
    ) -> HandleResult {
        // This special handler only exists because if labeled-response and echo-message
        // are both enabled, and the source and target of the message are the same user,
        // then they need to be notified of it twice, once with the response label and once
        // without. This is called out explicitly in the labeled-response spec, and appears
        // to exist solely to make my life difficult.

        let net = self.network();
        let message = net.message(msg.message)?;

        if let HistoricMessageSourceId::User(source) = &msg.source {
            if let HistoricMessageTargetId::User(target) = &msg.target {
                // Source and target are both users. Check for self-message with the awkward caps
                if source == target {
                    // We handle this as a special case.

                    let source = net.historic_user(*source)?;
                    let target = net.historic_user(*target)?;

                    let message = message::Message::new(
                        source,
                        target,
                        message.message_type(),
                        message.text(),
                    )
                    .with_tags_from(update, &net);

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

        self.send_now(update, &sink, update)
    }

    fn handle_new_user_connection(&self, detail: &update::NewUserConnection) -> HandleResult {
        let net = self.node.network();
        let user = net.user(*detail.user.user())?;

        if let Ok(connection) = self
            .connections
            .read()
            .get_user_connection(detail.connection)
        {
            // `register_new_user` doesn't set the user ID on the connection; it remains a pre-client until
            // we see the registration events come back through (i.e. here)
            connection.set_user(user.id(), detail.connection);

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

    fn handle_services_update(&self, _detail: &update::ServicesUpdate) -> HandleResult {
        let net = self.network();
        let new_state = net.current_services();

        match new_state {
            Some(state) => {
                let mut mechanisms = state.sasl_mechanisms().clone();
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
