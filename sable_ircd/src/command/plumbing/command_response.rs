use super::*;
use messages::{MessageSink, UntargetedNumeric, message};

/// A type-erased [`CommandResponseSink`]
pub struct CommandResponse<'a> (pub(in crate::command) Box<dyn CommandResponseSink + 'a>);

impl<'a> MessageSink for CommandResponse<'a> {
    fn send(&self, msg: OutboundClientMessage) {
        self.0.send(msg);
    }
    fn user_id(&self) -> Option<UserId> {
        self.0.user_id()
    }
}
impl<'a> CommandResponseSink for CommandResponse<'a> {
    fn notice(&self, text: &str) {
        self.0.notice(text);
    }
    fn numeric(&self, numeric: UntargetedNumeric) {
        self.0.numeric(numeric);
    }
}

/// Trait describing the ability to send responses to a command.
///
/// This essentially combines a [`MessageSink`] with knowledge of the source
/// and target parameters that should be injected into notices, numerics, and
/// the like.
pub trait CommandResponseSink : MessageSink + Send + Sync {
    /// Send the given text as a notice in response to this command
    fn notice(&self, text: &str);

    /// Send the given numeric response
    fn numeric(&self, numeric: UntargetedNumeric);
}

pub struct PlainCommandResponseSink<'a, Cmd, Sink> {
    command: &'a Cmd,
    sink: &'a Sink,
}

impl<'a, Cmd: Command, Sink: MessageSink> PlainCommandResponseSink<'a, Cmd, Sink> {
    pub (in crate::command) fn new(command: &'a Cmd, sink: &'a Sink) -> Self {
        Self {
            command,
            sink
        }
    }
}

impl<'a, Cmd: Command, Sink: MessageSink> CommandResponseSink for PlainCommandResponseSink<'a, Cmd, Sink> {
    fn notice(&self, text: &str) {
        self.send(message::Notice::new(self.command.response_source(), &self.command.source(), text));
    }

    fn numeric(&self, numeric: UntargetedNumeric) {
        self.send(numeric.format_for(self.command.response_source(), &self.command.source()))
    }
}

impl<'a, Cmd: Command, Sink: MessageSink> MessageSink for PlainCommandResponseSink<'a, Cmd, Sink> {
    fn send(&self, msg: OutboundClientMessage) {
        self.sink.send(msg);
    }

    fn user_id(&self) -> Option<UserId> {
        self.sink.user_id()
    }
}
