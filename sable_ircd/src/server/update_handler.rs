use messages::send_realtime::SendRealtimeItem;

use super::*;
use crate::errors::HandleResult;

impl ClientServer
{
    pub(super) fn handle_history_update(&mut self, update: NetworkHistoryUpdate) -> HandleResult
    {
        tracing::trace!(?update, "Got history update");

        match update
        {
            NetworkHistoryUpdate::NewEntry(entry_id) =>
            {
                let history = self.server.history();
                if let Some(entry) = history.get(entry_id)
                {
                    match &entry.details
                    {
                        NetworkStateChange::NewUser(detail) =>
                        {
                            let new_user = detail.clone();
                            drop(history);
                            self.handle_new_user(&new_user)?;
                        }
                        _ =>
                        {
                        }
                    }
                }
            }
            NetworkHistoryUpdate::NotifyUser(user_id, entry_id) =>
            {
                self.notify_user_update(user_id, entry_id)?;
            }
            NetworkHistoryUpdate::NotifyUsers(user_ids, entry_id) =>
            {
                for user_id in user_ids
                {
                    self.notify_user_update(user_id, entry_id)?;
                }
            }
        }

        Ok(())
    }

    fn notify_user_update(&self, user_id: UserId, entry_id: LogEntryId) -> HandleResult
    {
        for conn in self.connections.get_user(user_id)
        {
            let log = self.server.history();

            if let Some(entry) = log.get(entry_id)
            {
                entry.send_now(conn, entry, self)?;
            }
        }

        Ok(())
    }

    fn handle_new_user(&mut self, detail: &update::NewUser) -> HandleResult
    {
        let net = self.server.network();
        let user = net.user(detail.user.user.id)?;
        for connection in self.connections.get_user(user.id())
        {
            connection.set_user_id(user.id());

            connection.send(&numeric::Numeric001::new_for(&self.server.name().to_string(), &user.nick(), "test", &user.nick()));
            connection.send(&numeric::Numeric002::new_for(&self.server.name().to_string(), &user.nick(), self.server.name(), self.server.version()));
            for line in self.isupport.data().iter()
            {
                connection.send(&numeric::ISupport::new_for(&self.server.name().to_string(), &user.nick(), line));
            }

            connection.send(&message::Mode::new(&user, &user, &user.mode().format()));

            connection.send(&message::Notice::new(&self.server.name().to_string(), &user,
                    "The network is currently running in debug mode. Do not send any sensitive information such as passwords."));
        }
        Ok(())
    }

}