use super::{*, plumbing::Command};

/// Type alias for a boxed command context
pub type BoxCommand<'cmd> = Box<dyn Command + 'cmd>;

/// A command handler wrapper function. This is the type emitted by the `command_handler`
/// attribute macro
pub type CommandHandlerWrapper = for<'a> fn(BoxCommand<'a>) -> Option<AsyncHandler<'a>>;

/// A command handler registration. Constructed by the `command_handler` attribute macro.
pub struct CommandRegistration
{
    pub(super) command: &'static str,
    pub(super) dispatcher: Option<&'static str>,
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
    /// Construct a default `CommandDispatcher`.
    ///
    /// Handlers are populated via compile-time registration.
    pub fn new() -> Self
    {
        Self::construct(None)
    }

    /// Construct a dispatcher for the given category.
    ///
    /// This will dispatch to handlers registered with `#[command_handler("...", in = "<name>")]
    pub fn with_category(category_name: &str) -> Self
    {
        Self::construct(Some(category_name))
    }

    fn construct(category_name: Option<&str>) -> Self
    {
        let mut map = HashMap::new();

        for reg in inventory::iter::<CommandRegistration> {
            if reg.dispatcher == category_name
            {
                map.insert(reg.command.to_ascii_uppercase(), reg.handler);
            }
        }

        Self {
            handlers: map
        }
    }

    /// Look up and execute the handler function for to a given command.
    ///
    /// Returns `Some` if the handler is asynchronous and needs to be polled; `None` if the command
    /// was handled synchronously
    pub fn dispatch_command<'cmd>(&self, command: impl Command + 'cmd) -> Option<AsyncHandler<'cmd>>
    {
        let command: BoxCommand<'cmd> = Box::new(command);

        match self.handlers.get(&command.command().to_ascii_uppercase())
        {
            Some(handler) =>
            {
                handler(command)
            }
            None =>
            {
                command.notify_error(CommandError::CommandNotFound(command.command().to_owned()));
                None
            }
        }
    }
}

