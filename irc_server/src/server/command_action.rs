use super::*;

impl Server
{
    pub(super) fn apply_action(&mut self, action: CommandAction)
    {
        match action {
            CommandAction::RegisterClient(id) => {
                let mut should_add_user = None;
                let mut actions: Vec<(ObjectId, EventDetails)> = Vec::new();
                if let Ok(conn) = self.connections.get_mut(id)
                {
                    if let Some(pre_client_rc) = &conn.pre_client
                    {
                        // We don't delete the preclient here, because it's possible the event will fail to apply
                        // if someone else takes the nickname in between
                        let pre_client = pre_client_rc.borrow();
                        let new_user_id = self.id_generator.next_user();
                        let new_user_mode_id = self.id_generator.next_user_mode();

                        let details = event::details::NewUserMode {
                            mode: UserModeSet::new()
                        };
                        actions.push((new_user_mode_id.into(), details.into()));

                        let details = event::details::NewUser {
                            nickname: pre_client.nick.as_ref().unwrap().clone(),
                            username: pre_client.user.as_ref().unwrap().clone(),
                            visible_hostname: pre_client.hostname.as_ref().unwrap().clone(),
                            realname: pre_client.realname.as_ref().unwrap().clone(),
                            mode_id: new_user_mode_id,
                            server: self.my_id,
                        };
                        actions.push((new_user_id.into(), details.into()));

                        should_add_user = Some((new_user_id, id));
                    }
                }

                if let Some((user_id, conn_id)) = should_add_user
                {
                    self.connections.add_user(user_id, conn_id);
                }
                for act in actions
                {
                    self.submit_event(act.0, act.1);
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