use super::*;

impl Server
{
    pub(super) fn apply_action(&mut self, action: CommandAction)
    {
        match action {
            CommandAction::RegisterClient(id) => {
                let should_add_user = if let Ok(conn) = self.connections.get_mut(id)
                {
                    if let Some(pre_client_rc) = conn.pre_client.take()
                    {
                        let pre_client = pre_client_rc.into_inner();
                        let new_user_id = self.user_idgen.next();
                        let register_event = self.eventlog.create(
                                                        new_user_id, 
                                                        event::details::NewUser {
                                                            nickname: pre_client.nick.unwrap(),
                                                            username: pre_client.user.unwrap(),
                                                            visible_hostname: "example.com".to_string(),
                                                            realname: pre_client.realname.unwrap(),
                                                        }
                                                    );
                        self.eventlog.add(register_event);

                        Some((new_user_id, conn.id()))
                    } else { None }
                } else { None };

                if let Some((user_id, conn_id)) = should_add_user
                {
                    self.connections.add_user(user_id, conn_id);
                }
            },
            CommandAction::DisconnectUser(user_id) => {
                self.connections.remove_user(user_id);
            }
            CommandAction::StateChange(event) => {
                self.eventlog.add(event);
            }
        }
    }
}