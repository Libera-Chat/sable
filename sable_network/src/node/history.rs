use itertools::Itertools;

use super::*;

impl<Policy: crate::policy::PolicyService> NetworkNode<Policy> {
    pub(super) fn notify_user(&self, user_id: UserId, entry_id: LogEntryId) {
        self.history_log
            .read_recursive()
            .add_entry_for_user(user_id, entry_id);
        if let Err(e) = self
            .subscriber
            .send(NetworkHistoryUpdate::NotifyUser(user_id, entry_id))
        {
            tracing::error!("Error notifying subscriber of log update: {}", e);
        }
    }

    pub(super) fn notify_users(
        &self,
        user_ids: impl IntoIterator<Item = UserId>,
        entry_id: LogEntryId,
    ) {
        let log = self.history_log.read_recursive();

        let user_ids = user_ids.into_iter().collect_vec();

        for user in &user_ids {
            log.add_entry_for_user(*user, entry_id);
        }

        if let Err(e) = self
            .subscriber
            .send(NetworkHistoryUpdate::NotifyUsers(user_ids, entry_id))
        {
            tracing::error!("Error notifying subscriber of log update: {}", e);
        }
    }

    pub(super) fn notify_channel_members(
        &self,
        channel: &wrapper::Channel,
        entry: &HistoryLogEntry,
    ) {
        let users = channel.members().map(|m| m.user_id());
        self.notify_users(users, entry.id);
    }

    pub(super) fn notify_channel_members_where(
        &self,
        channel: &wrapper::Channel,
        entry: &HistoryLogEntry,
        predicate: impl Fn(&wrapper::Membership) -> bool,
    ) {
        self.notify_users(
            channel.members().filter(predicate).map(|m| m.user_id()),
            entry.id,
        );
    }
}
