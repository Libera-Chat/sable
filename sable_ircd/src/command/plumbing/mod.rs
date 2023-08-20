use client_listener::ConnectionId;
use messages::OutboundClientMessage;
use sable_network::prelude::*;
use crate::{
    server::ClientServer,
    command::CommandError, messages, client::ClientConnection,
};
use std::{
    sync::Arc,
    future::Future,
};

use super::{CommandSource, CommandResult};

pub trait Command : Send + Sync
{
    /// Return a `CommandSource` describing the originating user or connection
    fn source(&self) -> CommandSource<'_>;

    /// The command that was issued
    fn command(&self) -> &str;

    /// The arguments supplied to the command
    fn args(&self) -> ArgListIter;

    /// Access the [`ClientServer`]
    fn server(&self) -> &Arc<ClientServer>;
    /// Access the network state applicable to this command handler
    fn network(&self) -> &Arc<Network>;

    /// Notify the user of an error
    fn notify_error(&self, err: CommandError);

    /// Send a message in response to this command, to the user that originated it
    fn response(&self, message: OutboundClientMessage);

    /// Retrieve the underlying connection ID
    fn connection_id(&self) -> ConnectionId;

    /// Access the underlying connection object
    fn connection(&self) -> &ClientConnection;

    /// The source from which responses to this command should be sent
    fn response_source(&self) -> &dyn messages::MessageSource;
}

impl<T: Command + ?Sized> messages::MessageSink for T
{
    fn send(&self, msg: OutboundClientMessage)
    {
        self.response(msg)
    }

    fn user_id(&self) -> Option<UserId> {
        match self.source()
        {
            CommandSource::User(u) => Some(u.id()),
            CommandSource::PreClient(_) => None
        }
    }
}

pub(crate) fn call_handler<'a, Amb, Pos>(ctx: &'a dyn Command, handler: &impl HandlerFn<'a, Amb, Pos>, args: ArgListIter<'a>) -> CommandResult
{
    handler.call(ctx, args)
}

pub(crate) fn call_handler_async<'ctx, 'handler, Amb, Pos>(ctx: &'ctx dyn Command,
                                                       handler: &'handler impl AsyncHandlerFn<'ctx, Amb, Pos>,
                                                       args: ArgListIter<'ctx>
                                            ) -> impl Future<Output=CommandResult> + Send + Sync + 'ctx
    where 'handler: 'ctx
{
    handler.call(ctx, args)
}

mod command_ext;
pub use command_ext::*;

mod argument_list;
pub use argument_list::*;

mod argument_type;
pub use argument_type::*;

mod conditional_argument_types;
pub use conditional_argument_types::*;

mod argument_wrappers;
pub use argument_wrappers::*;

mod source_types;
pub use source_types::*;

mod target_types;
pub use target_types::*;

mod handler;
pub use handler::*;
