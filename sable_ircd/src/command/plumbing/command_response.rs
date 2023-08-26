use crate::capability::ClientCapability;

use super::*;
use messages::{
    batch::{BatchBuilder, LazyMessageBatch, MessageBatch},
    message, MessageSink, MessageSinkExt, MessageTarget, OutboundMessageTag, UntargetedNumeric,
};

/// Trait describing the ability to send responses to a command.
///
/// This essentially combines a [`MessageSink`] with knowledge of the source
/// and target parameters that should be injected into notices, numerics, and
/// the like.
pub trait CommandResponse: MessageSink + Send + Sync {
    /// Send the given text as a notice in response to this command
    fn notice(&self, text: &str);

    /// Send the given numeric response
    fn numeric(&self, numeric: UntargetedNumeric);
}

pub struct PlainResponseSink<Sink> {
    // We store these as strings to avoid reference cycles when holding a
    // response sink inside a ClientCommand
    response_source: String,
    response_target: String,
    sink: Arc<Sink>,
}

impl<Sink: MessageSink> PlainResponseSink<Sink> {
    pub(in crate::command) fn new(
        response_source: String,
        response_target: String,
        sink: Arc<Sink>,
    ) -> Self {
        Self {
            response_source,
            response_target,
            sink,
        }
    }
}

impl<Sink: MessageSink> CommandResponse for PlainResponseSink<Sink> {
    fn notice(&self, text: &str) {
        self.send(message::Notice::new(
            &self.response_source,
            &self.response_target,
            text,
        ));
    }

    fn numeric(&self, numeric: UntargetedNumeric) {
        self.send(numeric.format_for(&self.response_source, &self.response_target))
    }
}

impl<Sink: MessageSink> MessageSink for PlainResponseSink<Sink> {
    fn send(&self, msg: OutboundClientMessage) {
        self.sink.send(msg);
    }

    fn user_id(&self) -> Option<UserId> {
        self.sink.user_id()
    }
}

/// A response sink that supports the labeled-response capability
pub struct LabeledResponseSink<Sink: MessageSink> {
    // We store these as strings to avoid reference cycles when holding a
    // response sink inside a ClientCommand
    response_source: String,
    response_target: String,
    // The raw target and a copy of the label are needed to send ACK at the end
    raw_target: Arc<Sink>,
    label_tag: OutboundMessageTag,
    batch: LazyMessageBatch<Arc<Sink>>,
}

impl<'a, Sink: MessageSink> LabeledResponseSink<Sink> {
    pub(in crate::command) fn new(
        response_source: String,
        response_target: String,
        sink: Arc<Sink>,
        label: String,
    ) -> Self {
        let label_tag =
            OutboundMessageTag::new("label", Some(label), ClientCapability::LabeledResponse);

        let batch = Arc::clone(&sink)
            .into_batch("labeled-response", ClientCapability::LabeledResponse)
            .with_tag(label_tag.clone())
            .delay_start();

        Self {
            response_source,
            response_target,
            raw_target: sink,
            label_tag,
            batch,
        }
    }
}

impl<Sink: MessageSink> CommandResponse for LabeledResponseSink<Sink> {
    fn notice(&self, text: &str) {
        self.send(message::Notice::new(
            &self.response_source,
            &self.response_target,
            text,
        ));
    }

    fn numeric(&self, numeric: UntargetedNumeric) {
        self.send(numeric.format_for(&self.response_source, &self.response_target))
    }
}

impl<Sink: MessageSink> MessageSink for LabeledResponseSink<Sink> {
    fn send(&self, msg: OutboundClientMessage) {
        self.batch.send(msg);
    }

    fn user_id(&self) -> Option<UserId> {
        self.batch.user_id()
    }
}

impl<Sink: MessageSink> Drop for LabeledResponseSink<Sink> {
    fn drop(&mut self) {
        if self.batch.is_empty() {
            let msg = message::Ack::new(&self.response_source).with_tag(self.label_tag.clone());
            self.raw_target.send(msg);
        }
    }
}
