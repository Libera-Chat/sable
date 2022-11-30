use super::*;

use parking_lot::RwLockUpgradableReadGuard;

impl ClientServer
{
    pub(super) async fn apply_action(&self, action: CommandAction)
    {
        match action {
            CommandAction::RegisterClient(id) =>
            {
                let mut should_add_user = None;
                let connections = self.connections.upgradable_read();
                if let Ok(conn) = connections.get(id)
                {
                    {
                        if ! self.check_user_access(self, &*self.network(), &*conn)
                        {
                            RwLockUpgradableReadGuard::upgrade(connections).remove(id);
                            return;
                        }
                    }

                    if let Some(pre_client) = conn.pre_client()
                    {
                        // We don't delete the preclient here, because it's possible the event will fail to apply
                        // if someone else takes the nickname in between
                        let new_user_id = self.ids().next_user();

                        let mut umodes = UserModeSet::new();
                        if conn.connection.is_tls() {
                            umodes |= UserModeFlag::TlsConnection;
                        }

                        let details = event::details::NewUser {
                            nickname: *pre_client.nick.get().unwrap(),
                            username: *pre_client.user.get().unwrap(),
                            visible_hostname: *pre_client.hostname.get().unwrap(),
                            realname: pre_client.realname.get().unwrap().clone(),
                            mode: state::UserMode::new(umodes),
                            server: self.server.id(),
                            account: None,
                        };
                        self.server.submit_event(new_user_id, details);

                        should_add_user = Some((new_user_id, id));
                    }
                }

                if let Some((user_id, conn_id)) = should_add_user
                {
                    RwLockUpgradableReadGuard::upgrade(connections).add_user(user_id, conn_id);
                }
            }

            CommandAction::AttachToUser(connection_id, user_id) =>
            {
                // This operation will almost always require the write lock, so just get it immediately
                let mut connections = self.connections.write();
                if let Ok(conn) = connections.get(connection_id)
                {
                    conn.set_user_id(user_id);

                    connections.add_user(user_id, connection_id);
                }
            }

            CommandAction::UpdateConnectionCaps(conn_id, new_caps) =>
            {
                if let Ok(connection) = self.connections.get(conn_id)
                {
                    connection.capabilities.reset(new_caps);
                }
            }

            CommandAction::DisconnectUser(user_id) =>
            {
                self.connections.write().remove_user(user_id);
            }

            CommandAction::StateChange(id, detail) =>
            {
                self.server.submit_event(id, detail);
            }
        }
    }
}