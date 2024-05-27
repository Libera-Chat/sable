use super::Network;
use crate::network::event::*;
use crate::network::update::*;
use crate::prelude::*;

impl Network {
    pub(super) fn introduce_services(
        &mut self,
        target: ServerId,
        event: &Event,
        update: &IntroduceServices,
        updates: &dyn NetworkUpdateReceiver,
    ) {
        self.current_services = Some(state::ServicesData {
            server_id: target,
            sasl_mechanisms: update.sasl_mechanisms.clone(),
        });

        updates.notify(update::ServicesUpdate {}, event);
    }

    pub(super) fn update_account(
        &mut self,
        target: AccountId,
        _event: &Event,
        update: &AccountUpdate,
        _updates: &dyn NetworkUpdateReceiver,
    ) {
        if let Some(data) = &update.data {
            self.accounts.insert(target, data.clone());
        } else {
            // None here means deletion
            self.accounts.remove(&target);
        }
    }

    pub(super) fn update_nick_registration(
        &mut self,
        target: NickRegistrationId,
        _event: &Event,
        update: &NickRegistrationUpdate,
        _updates: &dyn NetworkUpdateReceiver,
    ) {
        if let Some(data) = &update.data {
            self.nick_registrations.insert(target, data.clone());
        } else {
            // None here means deletion
            self.nick_registrations.remove(&target);
        }
    }

    pub(super) fn update_channel_registration(
        &mut self,
        target: ChannelRegistrationId,
        _event: &Event,
        update: &ChannelRegistrationUpdate,
        _updates: &dyn NetworkUpdateReceiver,
    ) {
        if let Some(data) = &update.data {
            self.channel_registrations.insert(target, data.clone());
        } else {
            // None here means deletion
            self.channel_registrations.remove(&target);
        }
    }

    pub(super) fn update_channel_access(
        &mut self,
        target: ChannelAccessId,
        _event: &Event,
        update: &ChannelAccessUpdate,
        _updates: &dyn NetworkUpdateReceiver,
    ) {
        if let Some(data) = &update.data {
            self.channel_accesses.insert(target, data.clone());
        } else {
            // None here means deletion
            self.channel_accesses.remove(&target);
        }
    }

    pub(super) fn update_channel_role(
        &mut self,
        target: ChannelRoleId,
        _event: &Event,
        update: &ChannelRoleUpdate,
        _updates: &dyn NetworkUpdateReceiver,
    ) {
        if let Some(data) = &update.data {
            self.channel_roles.insert(target, data.clone());
        } else {
            // None here means deletion
            self.channel_roles.remove(&target);
        }
    }

    pub(super) fn user_login(
        &mut self,
        target: UserId,
        event: &Event,
        update: &UserLogin,
        updates: &dyn NetworkUpdateReceiver,
    ) {
        // Get the info we'll need later, before mutably borrowing self
        let accounts = &self.accounts;

        let Some(user) = self.users.get_mut(&target) else {
            return;
        };

        let old_account = user.account;
        let new_account = update.account;

        user.account = update.account;

        self.historic_users.update_account(
            user,
            event.timestamp,
            new_account
                .as_ref()
                .and_then(|id| accounts.get(id).map(|a| a.name)),
        );

        let user = user.clone();

        let update = update::UserLoginChange {
            user: self.translate_historic_user_id(&user),
            old_account,
            new_account,
        };

        updates.notify(update, event);
    }
}
