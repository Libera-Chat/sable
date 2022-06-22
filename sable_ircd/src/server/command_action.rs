use super::*;

impl ClientServer
{
    pub(super) async fn apply_action(&mut self, action: CommandAction)
    {
        match action {
            CommandAction::RegisterClient(id) =>
            {
                let mut should_add_user = None;
                if let Ok(conn) = self.connections.get(id)
                {
                    {
                        if ! self.policy().check_user_access(self, &*self.network(), conn)
                        {
                            self.connections.remove(id);
                            return;
                        }
                    }

                    if let Some(pre_client_rc) = &conn.pre_client
                    {
                        // We don't delete the preclient here, because it's possible the event will fail to apply
                        // if someone else takes the nickname in between
                        let pre_client = pre_client_rc.borrow();
                        let new_user_id = self.ids().next_user();

                        let mut umodes = UserModeSet::new();
                        if conn.connection.is_tls() {
                            umodes |= UserModeFlag::TlsConnection;
                        }

                        let details = event::details::NewUser {
                            nickname: *pre_client.nick.as_ref().unwrap(),
                            username: *pre_client.user.as_ref().unwrap(),
                            visible_hostname: *pre_client.hostname.as_ref().unwrap(),
                            realname: pre_client.realname.as_ref().unwrap().clone(),
                            mode: state::UserMode::new(umodes),
                            server: self.server.id(),
                        };
                        self.server.submit_event(new_user_id, details);

                        should_add_user = Some((new_user_id, id));
                    }
                }

                if let Some((user_id, conn_id)) = should_add_user
                {
                    self.connections.add_user(user_id, conn_id);
                }
            }

            CommandAction::UpdateConnectionCaps(conn_id, new_caps) =>
            {
                if let Ok(connection) = self.connections.get_mut(conn_id)
                {
                    connection.capabilities = new_caps;
                }
            }

            CommandAction::DisconnectUser(user_id) =>
            {
                self.connections.remove_user(user_id);
            }

            CommandAction::StateChange(id, detail) =>
            {
                self.server.submit_event(id, detail);
            }
        }
    }
}