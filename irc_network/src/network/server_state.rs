use super::*;

impl Network
{
    pub(super) fn new_server(&mut self, target: ServerId, event: &Event, detail: &details::NewServer)
    {
        if let Some(existing_epoch) = self.servers.get(&target).map(|s| s.epoch)
        {
            if existing_epoch != detail.epoch
            {
                self.delete_server(target);
            }
        }

        let server = state::Server {
            id: target,
            epoch: detail.epoch,
            name: detail.name.clone(),
            last_ping: detail.ts,
            introduced_by: event.id,
        };

        self.servers.insert(target, server);
    }

    pub(super) fn server_ping(&mut self, target: ServerId, _event: &Event, detail: &details::ServerPing)
    {
        if let Some(server) = self.servers.get_mut(&target)
        {
            server.last_ping = detail.ts
        }
    }

    pub(super) fn server_quit(&mut self, target: ServerId, _event: &Event, detail: &details::ServerQuit)
    {
        if let Some(server) = self.servers.get(&target) {
            if server.introduced_by != detail.introduced_by
            {
                return;
            }
        }
        self.delete_server(target);
    }

    fn delete_server(&mut self, target: ServerId)
    {
        if self.servers.remove(&target).is_some()
        {
            let mut users_to_remove = Vec::new();

            for u in self.users
                         .iter()
                         .filter(|&(_,v)| v.server == target)
                         .map(|(k,_)| *k)
            {
                users_to_remove.push(u);
            }

            for u in users_to_remove
            {
                self.memberships.retain(|_,v| v.user != u);
                self.users.remove(&u);
            }
        }
    }
}