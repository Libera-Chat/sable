use super::Network;
use crate::*;
use crate::event::*;
use crate::update::*;

impl Network
{
    fn translate_ban_setter(&self, id: UserId) -> String
    {
        if let Ok(user) = self.user(id)
        {
            format!("{}!{}@{}{{{}}}", user.nick(), user.user(), user.visible_host(), "opername")
        } else {
            "<unknown>".to_string()
        }
    }

    pub(super) fn new_kline(&mut self, target: NetworkBanId, event: &Event, details: &details::NewKLine, _updates: &dyn NetworkUpdateReceiver)
    {
        let kline = state::KLine {
            id: target,
            user: details.user.clone(),
            host: details.host.clone(),
            timestamp: event.timestamp,
            expires: event.timestamp + details.duration,
            setter_info: self.translate_ban_setter(details.setter),
            reason: details.user_reason.clone(),
            oper_reason: details.oper_reason.clone(),
        };

        self.klines.insert(target, kline);
    }

    pub(super) fn remove_kline(&mut self, target: NetworkBanId, _event: &Event, _details: &details::KLineRemoved, _updates: &dyn NetworkUpdateReceiver)
    {
        self.klines.remove(&target);
    }
}