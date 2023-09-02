use super::*;
use crate::network::update::*;
use wrapper::ObjectWrapper;

use parking_lot::RwLockReadGuard;
use thiserror::Error;

use std::collections::HashSet;

/// An error that could occur when handling a network state change
#[derive(Debug, Error)]
pub enum HandlerError {
    //    #[error("Internal error: {0}")]
    //    InternalError(String),
    #[error("Object lookup failed: {0}")]
    LookupError(#[from] LookupError),
    #[error("Mismatched object ID type")]
    WrongIdType(#[from] WrongIdTypeError),
}

pub type HandleResult = Result<(), HandlerError>;

impl<Policy: crate::policy::PolicyService> NetworkNode<Policy> {
    fn handle_away_change(
        &self,
        entry: &HistoryLogEntry,
        detail: &update::UserAwayChange,
    ) -> HandleResult {
        self.notify_user(detail.user.user.id, entry.id);
        Ok(())
    }

    fn handle_nick_change(
        &self,
        entry: &HistoryLogEntry,
        detail: &update::UserNickChange,
    ) -> HandleResult {
        // This fires after the nick change is applied to the network state, so we
        // have to construct the n!u@h string explicitly
        let net = self.network();
        let source = net.user(detail.user.id)?;
        let mut notified = HashSet::new();

        notified.insert(source.id());

        for membership in source.channels() {
            let chan = membership.channel()?;
            for m2 in chan.members() {
                notified.insert(m2.user_id());
            }
        }

        self.notify_users(notified, entry.id);

        Ok(())
    }

    fn handle_umode_change(
        &self,
        entry: &HistoryLogEntry,
        detail: &update::UserModeChange,
    ) -> HandleResult {
        self.notify_user(detail.user.user.id, entry.id);
        Ok(())
    }

    fn handle_user_quit(&self, entry: &HistoryLogEntry, detail: &update::UserQuit) -> HandleResult {
        let net = self.network();

        let mut notified = HashSet::new();

        for m1 in &detail.memberships {
            let m1: wrapper::Membership = ObjectWrapper::wrap(&*net, m1);
            for m2 in m1.channel()?.members() {
                notified.insert(m2.user_id());
            }
        }

        self.notify_users(notified, entry.id);
        Ok(())
    }

    fn handle_bulk_quit(
        &self,
        history_guard: &RwLockReadGuard<NetworkHistoryLog>,
        entry: &HistoryLogEntry,
        detail: &update::BulkUserQuit,
    ) -> HandleResult {
        for item in &detail.items {
            let new_entry = history_guard.add(
                NetworkStateChange::UserQuit(item.clone()),
                entry.source_event,
                entry.timestamp,
            );
            self.handle_user_quit(new_entry, item)?;
        }
        Ok(())
    }

    fn handle_channel_mode_change(
        &self,
        entry: &HistoryLogEntry,
        detail: &update::ChannelModeChange,
    ) -> HandleResult {
        let network = self.network();
        let channel = wrapper::Channel::wrap(&*network, &detail.channel);

        self.notify_channel_members(&channel, entry);

        Ok(())
    }

    fn handle_list_mode_added(
        &self,
        entry: &HistoryLogEntry,
        detail: &update::ListModeAdded,
    ) -> HandleResult {
        let network = self.network();
        let channel = wrapper::Channel::wrap(&*network, &detail.channel);

        self.notify_channel_members_where(&channel, entry, |m| {
            self.policy_service
                .should_see_list_change(m, detail.list_type)
        });

        Ok(())
    }

    fn handle_list_mode_removed(
        &self,
        entry: &HistoryLogEntry,
        detail: &update::ListModeRemoved,
    ) -> HandleResult {
        let network = self.network();
        let channel = wrapper::Channel::wrap(&*network, &detail.channel);

        self.notify_channel_members_where(&channel, entry, |m| {
            self.policy_service
                .should_see_list_change(m, detail.list_type)
        });

        Ok(())
    }

    fn handle_channel_topic(
        &self,
        entry: &HistoryLogEntry,
        detail: &update::ChannelTopicChange,
    ) -> HandleResult {
        let network = self.network();
        let channel = wrapper::Channel::wrap(&*network, &detail.channel);

        self.notify_channel_members(&channel, entry);

        Ok(())
    }

    fn handle_chan_perm_change(
        &self,
        entry: &HistoryLogEntry,
        detail: &update::MembershipFlagChange,
    ) -> HandleResult {
        let network = self.network();
        let channel = wrapper::Channel::wrap(&*network, &detail.channel);

        self.notify_channel_members(&channel, entry);

        Ok(())
    }

    fn handle_join(&self, entry: &HistoryLogEntry, detail: &update::ChannelJoin) -> HandleResult {
        let network = self.network();
        let channel = wrapper::Channel::wrap(&*network, &detail.channel);

        self.notify_channel_members(&channel, entry);

        Ok(())
    }

    fn handle_part(&self, entry: &HistoryLogEntry, detail: &update::ChannelPart) -> HandleResult {
        self.notify_user(detail.user.user.id, entry.id);

        let network = self.network();
        let channel = wrapper::Channel::wrap(&*network, &detail.channel);

        self.notify_channel_members(&channel, entry);

        Ok(())
    }

    fn handle_invite(
        &self,
        entry: &HistoryLogEntry,
        detail: &update::ChannelInvite,
    ) -> HandleResult {
        self.notify_user(detail.user.user.id, entry.id);

        Ok(())
    }

    fn handle_channel_rename(
        &self,
        entry: &HistoryLogEntry,
        detail: &update::ChannelRename,
    ) -> HandleResult {
        let network = self.network();
        let channel = wrapper::Channel::wrap(&*network, &detail.channel);

        self.notify_channel_members(&channel, entry);

        Ok(())
    }

    fn handle_new_message(
        &self,
        entry: &HistoryLogEntry,
        detail: &update::NewMessage,
    ) -> HandleResult {
        match &detail.target {
            update::HistoricMessageTarget::Channel(channel) => {
                let network = self.network();
                let channel = wrapper::Channel::wrap(&*network, channel);

                self.notify_channel_members(&channel, entry);
            }
            update::HistoricMessageTarget::User(user) => {
                self.notify_user(user.user.id, entry.id);
                // Users should always be allowed to see messages they send
                if let update::HistoricMessageSource::User(source) = &detail.source {
                    // However, if the source and target are the same, only notify them once -
                    // the client server code can handle duplication if required
                    if source.user.id != user.user.id {
                        self.notify_user(source.user.id, entry.id);
                    }
                }
            }
            update::HistoricMessageTarget::Unknown => (),
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn handle_new_server(
        &self,
        entry: &HistoryLogEntry,
        detail: &update::NewServer,
    ) -> HandleResult {
        tracing::trace!("Got new server");

        let net = self.network();
        let server = net.server(detail.server.id)?;

        self.sync_log().enable_server(*server.name(), server.id());

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn handle_server_quit(
        &self,
        entry: &HistoryLogEntry,
        detail: &update::ServerQuit,
    ) -> HandleResult {
        tracing::trace!("Got server quit");

        if detail.server.id == self.id() && detail.server.epoch == self.epoch() {
            // The network thinks we're no longer alive. Shut down to avoid desyncs
            panic!("Network thinks we're dead. Making it so");
        }

        self.sync_log()
            .disable_server(detail.server.name, detail.server.id, detail.server.epoch);

        Ok(())
    }

    fn report_audit_entry(
        &self,
        _entry: &HistoryLogEntry,
        _detail: &update::NewAuditLogEntry,
    ) -> HandleResult {
        Ok(())
    }

    fn handle_user_login(
        &self,
        _entry: &HistoryLogEntry,
        _detail: &update::UserLoginChange,
    ) -> HandleResult {
        // TODO: notify user(s)

        Ok(())
    }

    fn handle_services_update(
        &self,
        _entry: &HistoryLogEntry,
        _detail: &update::ServicesUpdate,
    ) -> HandleResult {
        Ok(())
    }
}

impl<Policy: crate::policy::PolicyService> NetworkUpdateReceiver for NetworkNode<Policy> {
    fn notify_update(&self, update: NetworkStateChange, source_event: &Event) {
        let history_guard = self.history_log.read();
        let entry = history_guard.add(update, source_event.id, source_event.timestamp);

        // Whatever the new entry is, we notify the subscriber of a new entry
        if let Err(e) = self
            .subscriber
            .send(NetworkHistoryUpdate::NewEntry(entry.id))
        {
            tracing::error!("Error sending log update to subscriber: {}", e);
        }

        // Then, once it's been notified of a new log entry, we process it to determine which users
        // should see it, add it to those users' personalised histories, and notify the subscriber
        // that the user should be shown the update
        use NetworkStateChange::*;
        let res = match &entry.details {
            NewUser(_details) => Ok(()),
            UserAwayChange(details) => self.handle_away_change(entry, details),
            UserNickChange(details) => self.handle_nick_change(entry, details),
            UserModeChange(details) => self.handle_umode_change(entry, details),
            UserQuit(details) => self.handle_user_quit(entry, details),
            BulkUserQuit(details) => self.handle_bulk_quit(&history_guard, entry, details),
            ChannelModeChange(details) => self.handle_channel_mode_change(entry, details),
            ListModeAdded(details) => self.handle_list_mode_added(entry, details),
            ListModeRemoved(details) => self.handle_list_mode_removed(entry, details),
            ChannelTopicChange(details) => self.handle_channel_topic(entry, details),
            ChannelJoin(details) => self.handle_join(entry, details),
            ChannelPart(details) => self.handle_part(entry, details),
            ChannelInvite(details) => self.handle_invite(entry, details),
            ChannelRename(details) => self.handle_channel_rename(entry, details),
            MembershipFlagChange(details) => self.handle_chan_perm_change(entry, details),
            NewMessage(details) => self.handle_new_message(entry, details),
            NewServer(details) => self.handle_new_server(entry, details),
            ServerQuit(details) => self.handle_server_quit(entry, details),
            NewAuditLogEntry(details) => self.report_audit_entry(entry, details),
            UserLoginChange(details) => self.handle_user_login(entry, details),
            ServicesUpdate(details) => self.handle_services_update(entry, details),
            // We don't need to do anything with EventComplete, just pass it along to the subscriber
            EventComplete(_) => Ok(()),
        };
        if let Err(e) = res {
            tracing::error!("Error ({}) handling state update {:?}", e, entry.details);
        }
    }
}
