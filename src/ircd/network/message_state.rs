use super::*;

impl Network
{
    pub(super) fn new_message(&mut self, target: MessageId, _event: &Event, details: &NewMessage)
    {
        let message = state::Message { id: target, source: details.source, target: details.target, text: details.text.clone() };
        self.messages.insert(target, message);
    }
}