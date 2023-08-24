use crate::{client::ClientConnection, command::CommandError, messages, server::ClientServer};
use client_listener::ConnectionId;
use messages::OutboundClientMessage;
use sable_network::prelude::*;
use std::{future::Future, sync::Arc};

use super::{CommandResult, CommandSource};

pub trait Command: Send + Sync {
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

    /// Retrieve a [`CommandResponse`] implementation which can receive responses
    /// to this command
    fn response_sink(&self) -> &dyn CommandResponse;

    /// Retrieve the underlying connection ID
    fn connection_id(&self) -> ConnectionId;

    /// Access the underlying connection object
    fn connection(&self) -> &ClientConnection;

    /// The source from which responses to this command should be sent
    fn response_source(&self) -> &dyn messages::MessageSource;
}

pub(crate) fn call_handler<'a, Amb, Pos>(
    ctx: &'a dyn Command,
    handler: &impl HandlerFn<'a, Amb, Pos>,
    args: ArgListIter<'a>,
) -> CommandResult {
    handler.call(ctx, args)
}

pub(crate) fn call_handler_async<'ctx, 'handler, Amb, Pos>(
    ctx: &'ctx dyn Command,
    handler: &'handler impl AsyncHandlerFn<'ctx, Amb, Pos>,
    args: ArgListIter<'ctx>,
) -> impl Future<Output = CommandResult> + Send + Sync + 'ctx
where
    'handler: 'ctx,
{
    handler.call(ctx, args)
}

mod command_ext;
pub use command_ext::*;

mod command_response;
pub use command_response::*;

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
