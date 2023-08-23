use super::*;

/// Trait describing an object to which a client protocol message can be sent
pub trait MessageSink : Send + Sync
{
    /// Send a protocol message to this sink
    fn send(&self, msg: OutboundClientMessage);

    /// Sometimes we need to know which, if any, user this will be sent to
    fn user_id(&self) -> Option<UserId>;
}

pub trait MessageSinkExt : MessageSink {
    /// Create a batch to be sent to this sink.
    ///
    /// Required parameters are the batch type as defined in the relevant IRCv3 specification,
    /// and the corresponding client capability. If a client does not have that capability
    /// enabled, then behaviour will fall back to sending those messages directly.
    fn batch(&self, batch_type: impl ToString,
                    capability: impl Into<ClientCapabilitySet>) -> batch::BatchBuilder<'_, Self> {
        batch::BatchBuilder::new(batch_type, capability, self)
    }
}

impl<T: MessageSink + ?Sized> MessageSinkExt for T { }
