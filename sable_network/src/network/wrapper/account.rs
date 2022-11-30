use crate::prelude::*;
use super::WrapIterator;

pub struct Account<'a> {
    network: &'a Network,
    data: &'a state::Account,
}

impl Account<'_>
{
    pub fn id(&self) -> AccountId
    {
        self.data.id
    }

    pub fn name(&self) -> Nickname
    {
        self.data.name
    }

    pub fn users(&self) -> impl Iterator<Item=wrapper::User>
    {
        let my_id = self.data.id;
        self.network.raw_users().filter(move |u| u.account == Some(my_id)).wrap(self.network)
    }
}

impl<'a> super::ObjectWrapper<'a> for Account<'a> {
    type Underlying = state::Account;

    fn wrap(net: &'a Network, data: &'a Self::Underlying) -> Self
    {
        Self{ network: net, data }
    }

    fn raw(&self) -> &'a Self::Underlying { self.data }
}

