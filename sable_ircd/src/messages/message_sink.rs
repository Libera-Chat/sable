use super::*;

/// Trait describing an object to which a client protocol message can be sent
pub trait MessageSink
{
    /// Send a protocol message to this sink
    fn send(&self, msg: OutboundClientMessage);

    /// Sometimes we need to know which, if any, user this will be sent to
    fn user_id(&self) -> Option<UserId>;

    /// Create a batch to be sent to this sink
    fn batch(&self, batch_type: impl ToString) -> batch::BatchBuilder<'_, Self> {
        batch::BatchBuilder::new(batch_type, self)
    }

    /// Create a named batch to be sent to this sink
    fn named_batch(&self, batch_type: impl ToString, name: impl ToString) -> batch::BatchBuilder<'_, Self> {
        batch::BatchBuilder::with_name(batch_type, name, self)
    }
}