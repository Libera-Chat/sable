use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use super::*;

/// Builds a batch and allows tags and parameters to be set before the batch is opened.
pub struct BatchBuilder<Underlying: MessageSink> {
    name: String,
    capability: ClientCapabilitySet,
    target: Underlying,
    batch_type: String,
    batch_args: Vec<String>,
    tags: Vec<OutboundMessageTag>,
}

fn random_batch_name() -> String {
    format!("{:x}", rand::random::<u128>())
}

impl<'a, Underlying: MessageSink> BatchBuilder<Underlying> {
    /// Construct a new builder with the given name
    pub(super) fn with_name(
        batch_type: impl ToString,
        capability: impl Into<ClientCapabilitySet>,
        name: impl ToString,
        target: Underlying,
    ) -> Self {
        Self {
            name: name.to_string(),
            capability: capability.into(),
            target,
            batch_type: batch_type.to_string(),
            batch_args: Default::default(),
            tags: Default::default(),
        }
    }

    /// Construct a new builder with a randomly generated name
    pub(super) fn new(
        batch_type: impl ToString,
        capability: impl Into<ClientCapabilitySet>,
        target: Underlying,
    ) -> Self {
        Self::with_name(batch_type, capability, random_batch_name(), target)
    }

    /// Add a tag to the batch
    pub fn with_tag(mut self, tag: OutboundMessageTag) -> Self {
        self.tags.push(tag);
        self
    }

    /// Add arguments
    pub fn with_arguments<'b>(
        mut self,
        args: impl IntoIterator<Item = &'b (impl ToString + 'b)>,
    ) -> Self {
        self.batch_args
            .extend(args.into_iter().map(ToString::to_string));
        self
    }

    /// Begin the batch. Executing this will send the `BATCH +<name>` message to
    /// open the batch and return the `MessageBatch` object to allow sending inside
    /// it. The `BATCH -<name>` closing message will be sent when that object drops.
    pub fn start(self) -> MessageBatch<Underlying> {
        let start_msg =
            message::BatchStart::new(&self.name, &self.batch_type, &self.batch_args.join(" "))
                .with_tags(&self.tags)
                .with_required_capabilities(self.capability);
        self.target.send(start_msg);
        MessageBatch {
            name: self.name,
            target: self.target,
            capability: self.capability,
        }
    }

    /// Build the batch, but don't send the start message yet. The batch start message
    /// will be sent before the first message written to the resulting [`LazyMessageBatch`].
    pub fn delay_start(self) -> LazyMessageBatch<Underlying> {
        let start_msg =
            message::BatchStart::new(&self.name, &self.batch_type, &self.batch_args.join(" "))
                .with_tags(&self.tags)
                .with_required_capabilities(self.capability);

        LazyMessageBatch::new(self.name, self.capability, self.target, start_msg)
    }
}

/// Represents an IRCv3 message batch.
///
/// A batch can be created referring to any existing message sink, and will act
/// as a sink itself. Messages written to the batch will be appropriately labelled
/// and wrapped before sending.
pub struct MessageBatch<Underlying: MessageSink> {
    name: String,
    capability: ClientCapabilitySet,
    target: Underlying,
}

impl<'a, Underlying: MessageSink> Drop for MessageBatch<Underlying> {
    fn drop(&mut self) {
        let end_msg =
            message::BatchEnd::new(&self.name).with_required_capabilities(self.capability);
        self.target.send(end_msg);
    }
}

impl<'a, Underlying: MessageSink> MessageSink for MessageBatch<Underlying> {
    fn send(&self, msg: OutboundClientMessage) {
        let tag = OutboundMessageTag::new("batch", Some(self.name.clone()), self.capability);
        let message = msg.with_tag(tag);
        self.target.send(message);
    }

    fn user_id(&self) -> Option<UserId> {
        self.target.user_id()
    }
}

/// A [`MessageBatch`], except that the batch start message is sent lazily on demand
pub struct LazyMessageBatch<Sink: MessageSink> {
    // We'd like to just hold a MessageBatch here, but that would unconditionally
    // send batch close on drop, even if we didn't open it
    name: String,
    capability: ClientCapabilitySet,
    target: Sink,
    start_msg: OutboundClientMessage,
    send_count: AtomicU32,
    sent_start: AtomicBool,
}

impl<Sink: MessageSink> LazyMessageBatch<Sink> {
    fn new(
        name: String,
        capability: ClientCapabilitySet,
        target: Sink,
        start_msg: OutboundClientMessage,
    ) -> Self {
        Self {
            name,
            capability,
            target,
            start_msg,
            send_count: AtomicU32::new(0),
            sent_start: AtomicBool::new(false),
        }
    }

    pub fn is_opened(&self) -> bool {
        self.sent_start.load(Ordering::Acquire)
    }
}

impl<'a, Underlying: MessageSink> Drop for LazyMessageBatch<Underlying> {
    fn drop(&mut self) {
        // Only send the batch end if we sent the start
        if self.is_opened() {
            let end_msg =
                message::BatchEnd::new(&self.name).with_required_capabilities(self.capability);
            self.target.send(end_msg);
        }
    }
}

impl<Sink: MessageSink> MessageSink for LazyMessageBatch<Sink> {
    fn send(&self, msg: OutboundClientMessage) {
        // Relaxed ordering is fine here; all that matters is that exactly one
        // call returns 0
        if self.send_count.fetch_add(1, Ordering::Relaxed) == 0 {
            // First message being sent. We need to open the batch
            self.target.send(self.start_msg.clone());
            self.sent_start.store(true, Ordering::Release);
        } else {
            // This isn't the first message, so we just need to wait until
            // the start message has definitely been sent before continuing
            while !self.sent_start.load(Ordering::Acquire) {
                // Nothing
            }
        }
        let tag = OutboundMessageTag::new("batch", Some(self.name.clone()), self.capability);
        let message = msg.with_tag(tag);
        self.target.send(message);
    }

    fn user_id(&self) -> Option<UserId> {
        self.target.user_id()
    }
}
