use sable_network::prelude::*;

use crate::{
    command::Command,
    messages::{MessageSink, UntargetedNumeric, message},
};

/// Extension trait adding some useful functionality to implementors of [`Command`]
pub trait CommandExt
{
    /// Send a notice to the source of the command
    fn notice(&self, text: impl ToString);
    /// Send a numeric to the source of the command
    fn numeric(&self, numeric: UntargetedNumeric);
    /// Submit a new network state event
    fn new_event(&self, target: impl Into<ObjectId>, detail: impl Into<EventDetails>);
}

impl<T: Command + ?Sized> CommandExt for T
{
    fn notice(&self, text: impl ToString)
    {
        let n = message::Notice::new(self.response_source(), &self.source(), &text.to_string());
        self.connection().send(n);
    }

    fn numeric(&self, numeric: UntargetedNumeric)
    {
        self.connection().send(numeric.format_for(self.response_source(), &self.source()));
    }

    fn new_event(&self, target: impl Into<ObjectId>, detail: impl Into<EventDetails>)
    {
        self.server().node().submit_event(target, detail);
    }
}