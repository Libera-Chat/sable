use super::Network;
use crate::prelude::*;
use crate::network::event::*;
use crate::network::update::*;

impl Network
{
    pub(super) fn new_ban(&mut self, target: NetworkBanId, event: &Event, details: &details::NewNetworkBan, _updates: &dyn NetworkUpdateReceiver)
    {
        let ban = state::NetworkBan {
            id: target,
            created_by: event.id,
            matcher: details.matcher.clone(),
            action: details.action.clone(),
            timestamp: details.timestamp,
            expires: details.expires,
            reason: details.reason.clone(),
            oper_reason: details.oper_reason.clone(),
            setter_info: details.setter_info.clone(),
        };

        if let Err(e) = self.network_bans.add(ban)
        {
            let ban = e.ban;
            if let Some(existing) = self.network_bans.get(&e.existing_id)
            {
                // Two separate bans with identical matchers - we choose one arbitrarily
                if ban.timestamp < existing.timestamp ||
                    ( ban.timestamp == existing.timestamp && ban.created_by < existing.created_by )
                {
                    self.network_bans.remove(existing.id);
                    if self.network_bans.add(ban).is_err()
                    {
                        todo!("handle this error, or at least report it");
                    }
                }
            }
        }
    }

    pub(super) fn remove_ban(&mut self, target: NetworkBanId, _event: &Event, _details: &details::RemoveNetworkBan, _updates: &dyn NetworkUpdateReceiver)
    {
        self.network_bans.remove(target);
    }
}