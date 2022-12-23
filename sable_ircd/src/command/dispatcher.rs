use super::{*, plumbing::CommandContext};

/// A command handler wrapper function. This is the type emitted by the `command_handler`
/// attribute macro
pub type CommandHandlerWrapper = fn(ClientCommand) -> Option<AsyncHandler>;

/// A command handler registration. Constructed by the `command_handler` attribute macro.
pub struct CommandRegistration
{
    pub(super) command: &'static str,
    pub(super) handler: CommandHandlerWrapper,
}

/// A command dispatcher. Collects registered command handlers and allows lookup by
/// command name.
pub struct CommandDispatcher
{
    handlers: HashMap<String, CommandHandlerWrapper>
}

inventory::collect!(CommandRegistration);

impl CommandDispatcher {
    /// Construct a `CommandDispatcher`.
    ///
    /// Handlers are populated via compile-time registration.
    pub fn new() -> Self
    {
        let mut map = HashMap::new();

        for reg in inventory::iter::<CommandRegistration> {
            map.insert(reg.command.to_ascii_uppercase(), reg.handler);
        }

        Self {
            handlers: map
        }
    }

    /// Look up and execute the handler function for to a given command.
    ///
    /// Returns `Some` if the handler is asynchronous and needs to be polled; `None` if the command
    /// was handled synchronously
    pub fn dispatch_command(&self, ctx: ClientCommand) -> Option<AsyncHandler>
    {
        match self.handlers.get(&ctx.command.to_ascii_uppercase())
        {
            Some(handler) =>
            {
                handler(ctx)
            }
            None =>
            {
                ctx.notify_error(CommandError::CommandNotFound(ctx.command.clone()));
                None
            }
        }
    }
}

