use super::*;
use crate::update::*;

impl Network
{
    pub(super) fn new_message(&mut self, target: MessageId, event: &Event, details: &details::NewMessage, updates: &dyn NetworkUpdateReceiver)
    {
        let message = state::Message {
            id: target,
            source: details.source,
            target: details.target,
            ts: event.timestamp,
            message_type: details.message_type,
            text: details.text.clone()
        };
        self.messages.insert(target, message.clone());

        updates.notify(update::NewMessage {
            message: message,
            source: self.translate_state_change_source(details.source.into()),
            target: self.translate_message_target(details.target.into())
        });
    }
}