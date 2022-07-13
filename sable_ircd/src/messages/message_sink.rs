use super::*;

pub trait MessageSink
{
    /// Send a protocol message to this sink
    fn send(&self, msg: &impl MessageTypeFormat);

    /// Sometimes we need to know which, if any, user this will be sent to
    fn user_id(&self) -> Option<UserId>;
}