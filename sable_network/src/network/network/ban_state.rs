use super::Network;
use crate::network::event::*;
use crate::network::update::*;
use crate::prelude::*;

impl Network {
    pub(super) fn new_ban(
        &mut self,
        target: NetworkBanId,
        event: &Event,
        details: &details::NewNetworkBan,
        _updates: &dyn NetworkUpdateReceiver,
    ) {
        let ban = state::NetworkBan {
            id: target,
            created_by: event.id,
            match_type: details.match_type,
            pattern: details.pattern.clone(),
            action: details.action,
            timestamp: details.timestamp,
            expires: details.expires,
            reason: details.reason.clone(),
            oper_reason: details.oper_reason.clone(),
            setter_info: details.setter_info.clone(),
        };

        self.network_bans.add(ban);
    }

    pub(super) fn remove_ban(
        &mut self,
        target: NetworkBanId,
        _event: &Event,
        _details: &details::RemoveNetworkBan,
        _updates: &dyn NetworkUpdateReceiver,
    ) {
        self.network_bans.remove(target);
    }
}
