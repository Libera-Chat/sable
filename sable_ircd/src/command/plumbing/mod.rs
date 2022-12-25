use sable_network::prelude::*;
use crate::{
    server::ClientServer,
    command::CommandError,
};
use std::{
    sync::Arc,
    future::Future,
};

use super::{CommandSource, CommandResult, ClientCommand, ArgumentList, ArgumentListIter};

pub trait CommandContext : Send + Sync
{
    /// Return a `CommandSource` describing the originating user or connection
    fn source(&self) -> CommandSource<'_>;

    /// The command that was issued
    fn command(&self) -> &ClientCommand;

    /// Access the [`ClientServer`]
    fn server(&self) -> &Arc<ClientServer>;
    /// Access the network state applicable to this command handler
    fn network(&self) -> &Arc<Network>;

    /// Notify the user of an error
    fn notify_error(&self, err: CommandError);
}

pub(crate) fn call_handler<'a, Amb, Pos>(ctx: &'a impl CommandContext, handler: &impl HandlerFn<'a, Amb, Pos>, args: &'a ArgumentList) -> CommandResult
{
    handler.call(ctx, args.iter())
}

pub(crate) fn call_handler_async<'ctx, 'handler, Amb, Pos>(ctx: &'ctx impl CommandContext,
                                                       handler: &'handler impl AsyncHandlerFn<'ctx, Amb, Pos>,
                                                       args: &'ctx ArgumentList
                                            ) -> impl Future<Output=CommandResult> + Send + Sync + 'ctx
    where 'handler: 'ctx
{
    handler.call(ctx, args.iter())
}

mod argument_type;
pub use argument_type::*;

mod conditional_argument_types;
pub use conditional_argument_types::*;

mod source_types;
pub use source_types::*;

mod target_types;
pub use target_types::*;

mod handler;
pub use handler::*;
