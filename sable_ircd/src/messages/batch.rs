use super::*;

/// Builds a batch and allows tags and parameters to be set before the batch is opened.
pub struct BatchBuilder<'a, Underlying: MessageSink + ?Sized + 'a> {
    batch: MessageBatch<'a, Underlying>,
    batch_type: String,
    batch_args: Vec<String>,
    tags: Vec<OutboundMessageTag>,
}

fn random_batch_name() -> String {
    format!("{:x}", rand::random::<u128>())
}

impl<'a, Underlying: MessageSink + ?Sized + 'a> BatchBuilder<'a, Underlying> {
    /// Construct a new builder with the given name
    pub(super) fn with_name(batch_type: impl ToString, name: impl ToString, target: &'a Underlying) -> Self {
        Self {
            batch: MessageBatch {
                name: name.to_string(),
                target,
            },
            batch_type: batch_type.to_string(),
            batch_args: Default::default(),
            tags: Default::default(),
        }
    }

    /// Construct a new builder with a randomly generated name
    pub(super) fn new(batch_type: impl ToString, target: &'a Underlying) -> Self {
        Self::with_name(batch_type, random_batch_name(), target)
    }

    /// Add a tag to the batch
    pub fn with_tag(mut self, tag: OutboundMessageTag) -> Self {
        self.tags.push(tag);
        self
    }

    /// Add arguments
    pub fn with_arguments<'b>(mut self, args: impl IntoIterator<Item=&'b (impl ToString + 'b)>) -> Self {
        self.batch_args.extend(args.into_iter().map(ToString::to_string));
        self
    }

    /// Begin the batch. Executing this will send the `BATCH +<name>` message to
    /// open the batch and return the `MessageBatch` object to allow sending inside
    /// it. The `BATCH -<name>` closing message will be sent when that object drops.
    pub fn start(self) -> MessageBatch<'a, Underlying> {
        let start_msg = message::BatchStart::new(&self.batch.name,
                                                 &self.batch_type,
                                                 &self.batch_args.join(" "))
                                            .with_tags(&self.tags)
                                            .with_required_capabilities(ClientCapability::Batch);
        self.batch.target.send(start_msg);
        self.batch
    }
}

/// Represents an IRCv3 message batch.
///
/// A batch can be created referring to any existing message sink, and will act
/// as a sink itself. Messages written to the batch will be appropriately labelled
/// and wrapped before sending.
pub struct MessageBatch<'a, Underlying: MessageSink + ?Sized + 'a> {
    name: String,
    target: &'a Underlying,
}

impl<'a, Underlying: MessageSink + ?Sized + 'a> Drop for MessageBatch<'a, Underlying> {
    fn drop(&mut self) {
        let end_msg = message::BatchEnd::new(&self.name)
                                        .with_required_capabilities(ClientCapability::Batch);
        self.target.send(end_msg);
    }
}

impl<'a, Underlying: MessageSink + ?Sized + 'a> MessageSink for MessageBatch<'a, Underlying> {
    fn send(&self, msg: OutboundClientMessage) {
        let tag = OutboundMessageTag::new("batch",
                                          Some(self.name.clone()),
                                          ClientCapability::Batch);
        let message = msg.with_tag(tag);
        self.target.send(message);
    }

    fn user_id(&self) -> Option<UserId> {
        self.target.user_id()
    }
}