use super::*;

impl Server
{
    pub(super) fn apply_action(&mut self, action: CommandAction)
    {
        match action {
            CommandAction::RegisterClient(id) => {
                let should_add_user = if let Ok(conn) = self.connections.get_mut(id)
                {
                    if let Some(pre_client_rc) = &conn.pre_client
                    {
                        // We don't delete the preclient here, because it's possible the event will fail to apply
                        // if someone else takes the nickname in between
                        let pre_client = pre_client_rc.borrow();
                        let new_user_id = self.user_idgen.next();
                        let details = event::details::NewUser {
                            nickname: pre_client.nick.as_ref().unwrap().clone(),
                            username: pre_client.user.as_ref().unwrap().clone(),
                            visible_hostname: Hostname::new("example.com".to_string()).unwrap(),
                            realname: pre_client.realname.as_ref().unwrap().clone(),
                        };

                        Some((new_user_id, conn.id(), new_user_id, details))
                    } else { None }
                } else { None };

                if let Some((user_id, conn_id, new_user_id, details)) = should_add_user
                {
                    self.connections.add_user(user_id, conn_id);
                    self.submit_event(new_user_id, details);
                }
            },
            CommandAction::DisconnectUser(user_id) => {
                self.connections.remove_user(user_id);
            }
            CommandAction::StateChange(id, detail) => {
                self.submit_event(id, detail);
            }
        }
    }
}