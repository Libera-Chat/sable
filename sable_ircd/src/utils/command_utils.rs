use sable_network::prelude::*;

use crate::{
    command_processor::ClientCommand,
    messages::{Numeric, message, MessageSink},
};

pub trait ClientCommandExt
{
    fn notice(&self, text: impl ToString);
    fn numeric(&self, numeric: impl Numeric);
    fn new_event(&self, target: impl Into<ObjectId>, detail: impl Into<EventDetails>);
}

impl ClientCommandExt for ClientCommand
{
    fn notice(&self, text: impl ToString)
    {
        let n = message::Notice::new(&self.server, &self.source(), &text.to_string());
        self.connection.send(&n);
    }

    fn numeric(&self, numeric: impl Numeric)
    {
        self.connection.send(&numeric.format_for(&self.server, &self.source()));
    }

    fn new_event(&self, target: impl Into<ObjectId>, detail: impl Into<EventDetails>)
    {
        self.server.server().submit_event(target, detail);
    }

}