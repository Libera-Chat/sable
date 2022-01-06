use super::*;
use crate::update::*;

impl Network
{
    pub(super) fn new_message(&mut self, target: MessageId, _event: &Event, details: &details::NewMessage, updates: &dyn NetworkUpdateReceiver)
    {
        let message = state::Message {
            id: target,
            source: details.source,
            target: details.target,
            message_type: details.message_type,
            text: details.text.clone()
        };
        self.messages.insert(target, message.clone());
        updates.notify(update::NewMessage {
            message: message.id
        });
    }
}