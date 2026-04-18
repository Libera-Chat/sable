use super::{plumbing::Command, *};

/// Type alias for a boxed command context
pub type BoxCommand<'cmd> = Box<dyn Command + 'cmd>;

/// A command handler wrapper function. This is the type emitted by the `command_handler`
/// attribute macro
pub type CommandHandlerWrapper = for<'a> fn(BoxCommand<'a>) -> Option<AsyncHandler<'a>>;

/// A command handler registration. Constructed by the `command_handler` attribute macro.
pub struct CommandRegistration {
    pub(super) command: &'static str,
    pub(super) dispatcher: Option<&'static str>,
    pub(super) handler: CommandHandlerWrapper,
}

/// A free-form help topic not tied to a command handler.
///
/// Use this for topics that describe concepts rather than commands — for example
/// channel modes, user modes, or any other subject a user might query with `HELP`.
///
/// Register a topic at the call site with [`inventory::submit!`]:
///
/// ```rust,ignore
/// inventory::submit!(HelpTopic {
///     topic: "CMODE_SECRET",
///     lines: &[
///         "CMODE_SECRET (+s)",
///         "",
///         "Marks the channel as secret. Secret channels are hidden from /LIST",
///         "and /WHOIS for users who are not members.",
///     ],
/// });
/// ```
///
/// Topics are looked up case-insensitively via [`CommandDispatcher::get_help_topic`].
pub struct HelpTopic {
    /// The name of the topic, matched case-insensitively against the argument to `HELP`.
    pub topic: &'static str,
    /// Lines of help text returned to the client.
    pub lines: &'static [&'static str],
}

/// A command dispatcher. Collects registered command handlers and allows lookup by
/// command name.
pub struct CommandDispatcher {
    handlers: HashMap<String, CommandHandlerWrapper>,
}

inventory::collect!(CommandRegistration);
inventory::collect!(HelpTopic);

impl CommandDispatcher {
    /// Construct a default `CommandDispatcher`.
    ///
    /// Handlers are populated via compile-time registration.
    pub fn new() -> Self {
        Self::construct(None)
    }

    /// Construct a dispatcher for the given category.
    ///
    /// This will dispatch to handlers registered with `#[command_handler("...", in = "&lt;name&gt;")]
    pub fn with_category(category_name: &str) -> Self {
        Self::construct(Some(category_name))
    }

    fn construct(category_name: Option<&str>) -> Self {
        let mut map = HashMap::new();

        for reg in inventory::iter::<CommandRegistration> {
            if reg.dispatcher == category_name {
                map.insert(reg.command.to_ascii_uppercase(), reg.handler);
            }
        }

        Self { handlers: map }
    }

    /// Look up and execute the handler function for to a given command.
    ///
    /// Returns `Some` if the handler is asynchronous and needs to be polled; `None` if the command
    /// was handled synchronously
    pub fn dispatch_command<'cmd>(
        &self,
        command: impl Command + 'cmd,
    ) -> Option<AsyncHandler<'cmd>> {
        let command: BoxCommand<'cmd> = Box::new(command);

        match self.handlers.get(&command.command().to_ascii_uppercase()) {
            Some(handler) => handler(command),
            None => {
                command.notify_error(CommandError::CommandNotFound(command.command().to_owned()));
                None
            }
        }
    }

    /// Look up a free-form [`HelpTopic`] by name (case-insensitive).
    ///
    /// Returns the topic's lines of text, or `None` if no topic with that name was registered.
    pub fn get_help_topic(&self, topic: &str) -> Option<&'static [&'static str]> {
        inventory::iter::<HelpTopic>
            .into_iter()
            .find(|t| t.topic.eq_ignore_ascii_case(topic))
            .map(|t| t.lines)
    }

    /// Iterate over all registered free-form [`HelpTopic`] entries.
    pub fn iter_help_topics(&self) -> impl Iterator<Item = &'static HelpTopic> {
        inventory::iter::<HelpTopic>.into_iter()
    }
}
