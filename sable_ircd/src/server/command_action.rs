use super::*;

use parking_lot::RwLockUpgradableReadGuard;

impl ClientServer {
    fn notify_access_error(&self, err: &user_access::AccessError, conn: &ClientConnection) {
        use user_access::AccessError::*;
        match err {
            Banned(reason) => {
                conn.send(make_numeric!(YoureBanned, &reason).format_for(self, &UnknownTarget));
            }
            SaslRequired(reason) => {
                if !reason.is_empty() {
                    conn.send(message::Notice::new(
                        self,
                        &UnknownTarget,
                        &format!("You must authenticate via SASL to use this server ({reason})"),
                    ));
                } else {
                    conn.send(message::Notice::new(
                        self,
                        &UnknownTarget,
                        "You must authenticate via SASL to use this server",
                    ));
                }
            }
            InternalError => {
                tracing::error!(?conn, "Internal error checking access");
                conn.send(message::Error::new("Internal error"));
            }
        }
    }

    fn register_new_user(&self, connection_id: ConnectionId) {
        let connections = self.connections.upgradable_read();
        if let Ok(conn) = connections.get(connection_id) {
            if let Some(pre_client) = conn.pre_client() {
                // First check whether they're attaching, as that's an easier operation
                if let Some(user_id) = pre_client.can_attach_to_user() {
                    let user_connection_id = self.ids().next();
                    let user_connection = event::details::NewUserConnection {
                        user: user_id,
                        hostname: *pre_client.hostname.get().unwrap(),
                        ip: conn.remote_addr(),
                        connection_time: sable_network::utils::now(),
                    };
                    self.node.submit_event(user_connection_id, user_connection);

                    let mut connections = RwLockUpgradableReadGuard::upgrade(connections);
                    connections.add_user(user_id, user_connection_id, conn.id());
                    conn.set_user(user_id, user_connection_id);
                    return;
                }

                // If we get this far, we're registering a new user
                if let Err(e) = self.check_user_access(&self.network(), &conn) {
                    self.notify_access_error(&e, conn.as_ref());
                    RwLockUpgradableReadGuard::upgrade(connections).remove(connection_id);
                    return;
                }

                let new_user_id = self.ids().next();

                if pre_client.can_register_new_user() {
                    let mut umodes = UserModeSet::new();
                    if conn.connection.is_tls() {
                        umodes |= UserModeFlag::TlsConnection;
                    }

                    let initial_connection_id = self.ids().next();
                    let initial_connection = event::details::NewUserConnection {
                        user: new_user_id,
                        hostname: *pre_client.hostname.get().unwrap(),
                        ip: conn.remote_addr(),
                        connection_time: sable_network::utils::now(),
                    };

                    let new_user = event::details::NewUser {
                        nickname: *pre_client.nick.get().unwrap(),
                        username: *pre_client.user.get().unwrap(),
                        visible_hostname: *pre_client.hostname.get().unwrap(),
                        realname: *pre_client.realname.get().unwrap(),
                        mode: state::UserMode::new(umodes),
                        server: self.node.id(),
                        account: pre_client.sasl_account.get().cloned(),
                        initial_connection: Some((initial_connection_id, initial_connection)),
                    };
                    self.node.submit_event(new_user_id, new_user);

                    RwLockUpgradableReadGuard::upgrade(connections).add_user(
                        new_user_id,
                        initial_connection_id,
                        connection_id,
                    );
                }
            }
        }
    }

    pub(super) async fn apply_action(&self, action: CommandAction) {
        match action {
            CommandAction::RegisterClient(connection_id) => {
                self.register_new_user(connection_id);
            }

            CommandAction::UpdateConnectionCaps(conn_id, new_caps) => {
                if let Ok(connection) = self.connections.get(conn_id) {
                    connection.capabilities.set_all(new_caps);
                }
            }

            CommandAction::DisconnectUser(user_id) => {
                self.connections.write().remove_user(user_id);
            }

            CommandAction::CloseConnection(conn_id) => {
                self.connections.write().remove(conn_id);
            }

            CommandAction::StateChange(id, detail) => {
                self.node.submit_event(id, detail);
            }
        }
    }
}
