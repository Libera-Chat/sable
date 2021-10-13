use super::*;

impl Server
{
    pub(super) fn apply_action(&mut self, action: CommandAction)
    {
        match action {
            CommandAction::RegisterClient(id) => {
                if let Some(conn) = self.client_connections.get_mut(&id)
                {
                    if let Some(pre_client_rc) = conn.pre_client.take()
                    {
                        let pre_client = pre_client_rc.into_inner();
                        let new_user_id = self.user_idgen.next();
                        let register_event = self.eventlog.create(
                                                        ObjectId::User(new_user_id), 
                                                        event::details::NewUser {
                                                            nickname: pre_client.nick.unwrap(),
                                                            username: pre_client.user.unwrap(),
                                                            visible_hostname: "example.com".to_string(),
                                                            realname: pre_client.realname.unwrap(),
                                                        }.into()
                                                    );
                        self.eventlog.add(register_event);

                        self.user_connections.insert(new_user_id, conn.id());
                    }
                }
            },
            CommandAction::StateChange(event) => {
                self.eventlog.add(event);
            }
        }
    }
}