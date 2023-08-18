use super::*;

/// Trait describing an object to which a client protocol message can be sent
pub trait MessageSink
{
    /// Send a protocol message to this sink
    fn send(&self, msg: &OutboundClientMessage);

    /// Sometimes we need to know which, if any, user this will be sent to
    fn user_id(&self) -> Option<UserId>;
}