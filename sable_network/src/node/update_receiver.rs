use super::*;
use crate::network::{
    update::*,
    wrapper::{WrappedMessage, WrappedUser},
};
use wrapper::ObjectWrapper;

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

pub type HandleResult = Result<Vec<UserId>, HandlerError>;

impl<Policy: crate::policy::PolicyService> NetworkNode<Policy> {
    fn handle_away_change(&self, detail: &update::UserAwayChange) -> HandleResult {
        let net = self.network();
        let source = net.user(detail.user.id())?;

        let mut notified = HashSet::new();

        // Notify the source user themselves, even if they are not in any channel
        notified.insert(detail.user.id());

        for m1 in source.channels() {
            let chan = m1.channel()?;
            for m2 in chan.members() {
                notified.insert(m2.user_id());
            }
        }

        Ok(notified.into_iter().collect())
    }

    fn handle_nick_change(&self, detail: &update::UserNickChange) -> HandleResult {
        let net = self.network();
        let source = net.user(detail.user.id())?;
        let mut notified = HashSet::new();

        notified.insert(source.id());

        for membership in source.channels() {
            let chan = membership.channel()?;
            for m2 in chan.members() {
                notified.insert(m2.user_id());
            }
        }

        Ok(notified.into_iter().collect())
    }

    fn handle_umode_change(&self, detail: &update::UserModeChange) -> HandleResult {
        Ok(vec![detail.user.id()])
    }

    fn handle_user_quit(&self, detail: &update::UserQuit) -> HandleResult {
        let net = self.network();

        let mut notified = HashSet::new();

        for m1 in &detail.memberships {
            let m1: wrapper::Membership = ObjectWrapper::wrap(&net, m1);
            for m2 in m1.channel()?.members() {
                notified.insert(m2.user_id());
            }
        }

        Ok(notified.into_iter().collect())
    }

    fn handle_channel_mode_change(&self, detail: &update::ChannelModeChange) -> HandleResult {
        let network = self.network();
        let channel = network.channel(detail.channel.id)?;

        Ok(channel.members().map(|m| m.user_id()).collect())
    }

    fn handle_list_mode_added(&self, detail: &update::ListModeAdded) -> HandleResult {
        let network = self.network();
        let channel = network.channel(detail.channel.id)?;

        Ok(channel
            .members()
            .filter_map(|m| {
                if self
                    .policy_service
                    .should_see_list_change(&m, detail.list_type)
                {
                    Some(m.user_id())
                } else {
                    None
                }
            })
            .collect())
    }

    fn handle_list_mode_removed(&self, detail: &update::ListModeRemoved) -> HandleResult {
        let network = self.network();
        let channel = network.channel(detail.channel.id)?;

        Ok(channel
            .members()
            .filter_map(|m| {
                if self
                    .policy_service
                    .should_see_list_change(&m, detail.list_type)
                {
                    Some(m.user_id())
                } else {
                    None
                }
            })
            .collect())
    }

    fn handle_channel_topic(&self, detail: &update::ChannelTopicChange) -> HandleResult {
        let network = self.network();
        let channel = network.channel(detail.channel.id)?;

        Ok(channel.members().map(|m| m.user_id()).collect())
    }

    fn handle_chan_perm_change(&self, detail: &update::MembershipFlagChange) -> HandleResult {
        let network = self.network();
        let channel = network.channel(detail.membership.channel)?;

        Ok(channel.members().map(|m| m.user_id()).collect())
    }

    fn handle_join(&self, detail: &update::ChannelJoin) -> HandleResult {
        let network = self.network();
        let channel = network.channel(detail.membership.channel)?;

        Ok(channel.members().map(|m| m.user_id()).collect())
    }

    fn handle_kick(&self, detail: &update::ChannelKick) -> HandleResult {
        let network = self.network();
        let channel = network.channel(detail.membership.channel)?;

        let mut users: Vec<_> = channel.members().map(|m| m.user_id()).collect();
        users.push(detail.membership.user);

        Ok(users)
    }

    fn handle_part(&self, detail: &update::ChannelPart) -> HandleResult {
        let network = self.network();
        let channel = network.channel(detail.membership.channel)?;

        let mut users: Vec<_> = channel.members().map(|m| m.user_id()).collect();
        users.push(detail.membership.user);

        Ok(users)
    }

    fn handle_invite(&self, detail: &update::ChannelInvite) -> HandleResult {
        Ok(vec![detail.invite.id.user()])
    }

    fn handle_channel_rename(&self, detail: &update::ChannelRename) -> HandleResult {
        let network = self.network();
        let channel = network.channel(detail.channel.id)?;

        Ok(channel.members().map(|m| m.user_id()).collect())
    }

    fn handle_new_message(&self, detail: &update::NewMessage) -> HandleResult {
        let network = self.network();
        let message = network.message(detail.message.id)?;

        Ok(match &message.target()? {
            wrapper::MessageTarget::Channel(channel) => {
                channel.members().map(|m| m.user_id()).collect()
            }
            wrapper::MessageTarget::User(user) => {
                // Users should always be allowed to see messages they send
                let mut result = vec![user.id()];
                if let Ok(source) = message.source() {
                    // However, if the source and target are the same, only notify them once -
                    // the client server code can handle re-duplication if required by echo-message
                    if source.id() != user.id() {
                        result.push(source.id());
                    }
                }
                result
            }
        })
    }

    #[tracing::instrument(skip(self))]
    fn handle_new_server(&self, detail: &update::NewServer) -> HandleResult {
        tracing::trace!("Got new server");

        self.sync_log()
            .enable_server(detail.server.name, detail.server.id);

        Ok(Vec::new())
    }

    #[tracing::instrument(skip(self))]
    fn handle_server_quit(&self, detail: &update::ServerQuit) -> HandleResult {
        tracing::trace!("Got server quit");

        if detail.server.id == self.id() && detail.server.epoch == self.epoch() {
            // The network thinks we're no longer alive. Shut down to avoid desyncs
            panic!("Network thinks we're dead. Making it so");
        }

        self.sync_log()
            .disable_server(detail.server.name, detail.server.id, detail.server.epoch);

        Ok(Vec::new())
    }

    fn report_audit_entry(&self, _detail: &update::NewAuditLogEntry) -> HandleResult {
        Ok(Vec::new())
    }

    fn handle_new_user_connection(&self, _detail: &update::NewUserConnection) -> HandleResult {
        Ok(Vec::new())
    }

    fn handle_user_connection_disconnected(
        &self,

        _detail: &update::UserConnectionDisconnected,
    ) -> HandleResult {
        Ok(Vec::new())
    }

    fn handle_user_login(&self, _detail: &update::UserLoginChange) -> HandleResult {
        // TODO: notify user(s)

        Ok(Vec::new())
    }

    fn handle_services_update(&self, _detail: &update::ServicesUpdate) -> HandleResult {
        Ok(Vec::new())
    }
}

impl<Policy: crate::policy::PolicyService> NetworkUpdateReceiver for NetworkNode<Policy> {
    fn notify_update(&self, update: NetworkStateChange, event: &Event) {
        let history_guard = self.history_log.read();
        let entry_id = history_guard.add(update.clone(), event.id, event.timestamp);

        // Then, once it's been notified of a new log entry, we process it to determine which users
        // should see it, add it to those users' personalised histories, and notify the subscriber
        // that the user should be shown the update
        use NetworkStateChange::*;
        let result = match &update {
            NewUser(_) => Ok(Vec::new()),
            UserAwayChange(detail) => self.handle_away_change(detail),
            UserNickChange(detail) => self.handle_nick_change(detail),
            UserModeChange(detail) => self.handle_umode_change(detail),
            NewUserConnection(detail) => self.handle_new_user_connection(detail),
            UserConnectionDisconnected(detail) => self.handle_user_connection_disconnected(detail),
            UserQuit(detail) => self.handle_user_quit(detail),
            ChannelModeChange(detail) => self.handle_channel_mode_change(detail),
            ListModeAdded(detail) => self.handle_list_mode_added(detail),
            ListModeRemoved(detail) => self.handle_list_mode_removed(detail),
            ChannelTopicChange(detail) => self.handle_channel_topic(detail),
            ChannelJoin(detail) => self.handle_join(detail),
            ChannelKick(detail) => self.handle_kick(detail),
            ChannelPart(detail) => self.handle_part(detail),
            ChannelInvite(detail) => self.handle_invite(detail),
            ChannelRename(detail) => self.handle_channel_rename(detail),
            MembershipFlagChange(detail) => self.handle_chan_perm_change(detail),
            NewMessage(detail) => self.handle_new_message(detail),
            NewServer(detail) => self.handle_new_server(detail),
            ServerQuit(detail) => self.handle_server_quit(detail),
            NewAuditLogEntry(detail) => self.report_audit_entry(detail),
            UserLoginChange(detail) => self.handle_user_login(detail),
            ServicesUpdate(detail) => self.handle_services_update(detail),
            // We don't need to do anything with EventComplete, just pass it along to the subscriber
            EventComplete(_) => Ok(Vec::new()),
        };
        let users_to_notify = match result {
            Err(e) => {
                tracing::error!("Error ({}) handling state update {:?}", e, update);
                return;
            }
            Ok(users) => users,
        };

        // Now that we know which users to notify, add to their logs and send it through to the subscriber
        if let Some(entry_id) = entry_id {
            for user in &users_to_notify {
                history_guard.add_entry_for_user(*user, entry_id);
            }
        }

        if let Err(e) = self.subscriber.send(NetworkHistoryUpdate {
            event: event.id,
            timestamp: event.timestamp,
            change: update,
            users_to_notify,
        }) {
            tracing::error!("Error sending log update to subscriber: {}", e);
        }
    }
}
