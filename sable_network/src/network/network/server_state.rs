use super::*;

impl Network {
    pub(super) fn new_server(
        &mut self,
        target: ServerId,
        event: &Event,
        detail: &details::NewServer,
        updates: &dyn NetworkUpdateReceiver,
    ) {
        if let Some(existing_epoch) = self.servers.get(&target).map(|s| s.epoch) {
            if existing_epoch != detail.epoch {
                self.delete_server(target, event, updates);
            }
        }

        let server = state::Server {
            id: target,
            epoch: detail.epoch,
            name: detail.name,
            last_ping: detail.ts,
            flags: detail.flags,
            version: detail.version.clone(),
        };

        self.servers.insert(target, server.clone());

        updates.notify(update::NewServer { server }, event);
    }

    pub(super) fn server_ping(
        &mut self,
        target: ServerId,
        _event: &Event,
        detail: &details::ServerPing,
        _updates: &dyn NetworkUpdateReceiver,
    ) {
        if let Some(server) = self.servers.get_mut(&target) {
            server.last_ping = detail.ts
        }
    }

    pub(super) fn server_quit(
        &mut self,
        target: ServerId,
        event: &Event,
        detail: &details::ServerQuit,
        updates: &dyn NetworkUpdateReceiver,
    ) {
        if let Some(server) = self.servers.get(&target) {
            if server.epoch != detail.epoch {
                return;
            }
        }
        self.delete_server(target, event, updates);
    }

    fn delete_server(
        &mut self,
        target: ServerId,
        event: &Event,
        updates: &dyn NetworkUpdateReceiver,
    ) {
        if let Some(removed) = self.servers.remove(&target) {
            // If the server being removed is the current services node, then we need to notify
            // that services has gone away
            if let Some(services) = &self.current_services {
                if removed.id == services.server_id {
                    updates.notify(update::ServicesUpdate { new_state: None }, event);
                }
            }

            // Collect all the user connections that were on the departing server
            let removed_connections: Vec<_> = self
                .user_connections
                .extract_if(|id, _conn| id.server() == target)
                .map(|(_, conn)| conn)
                .collect();

            // Identify the set of users associated with those connections
            let users_to_test: Vec<_> = removed_connections.iter().map(|conn| conn.user).collect();

            // Check which of those users aren't persistent, and quit them
            let mut quit_updates = Vec::new();

            for user_id in users_to_test {
                if matches!(self.users.get(&user_id), Some(user) if user.session_key.is_none()) {
                    if let Some(update) =
                        self.remove_user(user_id, "Server disconnecting".to_string())
                    {
                        quit_updates.push(update);
                    }
                }
            }

            updates.notify(update::ServerQuit { server: removed }, event);

            updates.notify(
                update::BulkUserQuit {
                    items: quit_updates,
                },
                event,
            );
        }
    }
}
