use super::Network;
use crate::prelude::*;
use crate::network::event::*;
use crate::network::update::*;

impl Network
{
    pub(super) fn new_ban(&mut self, target: NetworkBanId, _event: &Event, details: &details::NewNetworkBan, _updates: &dyn NetworkUpdateReceiver)
    {
        self.network_bans.add(details.data.clone());
    }

    pub(super) fn remove_ban(&mut self, target: NetworkBanId, _event: &Event, _details: &details::RemoveNetworkBan, _updates: &dyn NetworkUpdateReceiver)
    {
        self.network_bans.remove(target);
    }
}