use sable_network::prelude::*;

use crate::{
    command::CommandContext,
    messages::{Numeric, message, MessageSink},
};

pub trait ClientCommandExt
{
    fn notice(&self, text: impl ToString);
    fn numeric(&self, numeric: impl Numeric);
    fn new_event(&self, target: impl Into<ObjectId>, detail: impl Into<EventDetails>);
}

impl<T: CommandContext> ClientCommandExt for T
{
    fn notice(&self, text: impl ToString)
    {
        let n = message::Notice::new(self.server(), &self.source(), &text.to_string());
        self.command().connection.send(&n);
    }

    fn numeric(&self, numeric: impl Numeric)
    {
        self.command().connection.send(&numeric.format_for(self.server(), &self.source()));
    }

    fn new_event(&self, target: impl Into<ObjectId>, detail: impl Into<EventDetails>)
    {
        self.server().server().submit_event(target, detail);
    }
}