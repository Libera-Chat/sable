use super::Network;
use crate::prelude::*;
use crate::network::event::*;
use crate::network::update::*;

impl Network
{
    pub(super) fn update_account(&mut self, target: AccountId, _event: &Event, update: &AccountUpdate, _updates: &dyn NetworkUpdateReceiver)
    {
        if let Some(data) = &update.data
        {
            self.accounts.insert(target, data.clone());
        }
        else
        {
            // None here means deletion
            self.accounts.remove(&target);
        }
    }

    pub(super) fn update_nick_registration(&mut self, target: NickRegistrationId, _event: &Event, update: &NickRegistrationUpdate, _updates: &dyn NetworkUpdateReceiver)
    {
        if let Some(data) = &update.data
        {
            self.nick_registrations.insert(target, data.clone());
        }
        else
        {
            // None here means deletion
            self.nick_registrations.remove(&target);
        }
    }

    pub(super) fn update_channel_registration(&mut self, target: ChannelRegistrationId, _event: &Event, update: &ChannelRegistrationUpdate, _updates: &dyn NetworkUpdateReceiver)
    {
        if let Some(data) = &update.data
        {
            self.channel_registrations.insert(target, data.clone());
        }
        else
        {
            // None here means deletion
            self.channel_registrations.remove(&target);
        }
    }

    pub(super) fn update_channel_access(&mut self, target: ChannelAccessId, _event: &Event, update: &ChannelAccessUpdate, _updates: &dyn NetworkUpdateReceiver)
    {
        if let Some(data) = &update.data
        {
            self.channel_accesses.insert(target, data.clone());
        }
        else
        {
            // None here means deletion
            self.channel_accesses.remove(&target);
        }
    }
}