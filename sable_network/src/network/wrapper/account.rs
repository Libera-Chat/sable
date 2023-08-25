use super::WrapIterator;
use crate::prelude::*;

pub struct Account<'a> {
    network: &'a Network,
    data: &'a state::Account,
}

impl Account<'_> {
    pub fn id(&self) -> AccountId {
        self.data.id
    }

    pub fn name(&self) -> Nickname {
        self.data.name
    }

    pub fn users(&self) -> impl Iterator<Item = wrapper::User> {
        let my_id = self.data.id;
        self.network
            .raw_users()
            .filter(move |u| u.account == Some(my_id))
            .wrap(self.network)
    }

    pub fn channel_accesses(&self) -> impl Iterator<Item = wrapper::ChannelAccess> {
        let my_id = self.data.id;
        self.network
            .channel_accesses()
            .filter(move |a| a.id().account() == my_id)
    }

    pub fn has_access_in(&self, channel: ChannelRegistrationId) -> Option<wrapper::ChannelAccess> {
        let access_id = ChannelAccessId::new(self.data.id, channel);
        self.network.channel_access(access_id).ok()
    }

    pub fn fingerprints(&self) -> &Vec<String> {
        &&self.data.authorised_fingerprints
    }
}

impl<'a> super::ObjectWrapper<'a> for Account<'a> {
    type Underlying = state::Account;

    fn wrap(net: &'a Network, data: &'a Self::Underlying) -> Self {
        Self { network: net, data }
    }

    fn raw(&self) -> &'a Self::Underlying {
        self.data
    }
}
