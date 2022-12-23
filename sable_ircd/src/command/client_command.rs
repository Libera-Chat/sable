use super::{*, plumbing::CommandContext};
use sable_network::network::wrapper::ObjectWrapper;

/// Describes the possible types of connection that can invoke a command handler
pub enum CommandSource<'a>
{
    /// A client connection which has not yet completed registration
    PreClient(Arc<PreClient>),
    /// A client connection which is associated with a network user
    User(wrapper::User<'a>),
}

/// Internal representation of a `CommandSource`
enum InternalCommandSource
{
    PreClient(Arc<PreClient>),
    User(*const state::User),
}

/// A client command to be handled
pub struct ClientCommand
{
    /// The [`ClientServer`] instance
    pub server: Arc<ClientServer>,
    /// The connection from which the command originated
    pub connection: Arc<ClientConnection>,
    /// The network state as seen by this command handlers
    pub net: Arc<Network>,
    /// Details of the user associated with the connection
    source: InternalCommandSource,
    /// The command being executed
    pub command: String,
    /// Arguments supplied
    pub args: ArgumentList,
}

// Safety: this isn't automatically Send/Sync because of the raw pointer inside `InternalCommandSource`.
// It's safe, though, because that pointer points into an Arc<> held by the same `ClientCommand`.
unsafe impl Send for ClientCommand { }
unsafe impl Sync for ClientCommand { }

impl ClientCommand
{
    /// Construct a `ClientCommand`
    pub fn new(server: Arc<ClientServer>,
           connection: Arc<ClientConnection>,
           message: ClientMessage,
        ) -> Result<Self, CommandError>
    {
        let net = server.network();
        let source = Self::translate_message_source(&*net, &*connection)?;

        Ok(Self {
            server,
            connection,
            net,
            source,
            command: message.command,
            args: message.args.into(),
        })
    }

    fn translate_message_source(net: &Network, source: &ClientConnection) -> Result<InternalCommandSource, CommandError>
    {
        if let Some(user_id) = source.user_id()
        {
            let user_state = net.user(user_id)?.raw();
            Ok(InternalCommandSource::User(user_state))
        }
        else if let Some(pre_client) = source.pre_client()
        {
            Ok(InternalCommandSource::PreClient(pre_client))
        }
        else
        {
            Err(CommandError::unknown("Got message from neither preclient nor client"))
        }
    }

    pub fn response(&self, m: &impl messages::MessageTypeFormat)
    {
        self.connection.send(m);
    }
}

impl CommandContext for ClientCommand
{
    fn source(&self) -> CommandSource<'_>
    {
        match &self.source
        {
            InternalCommandSource::PreClient(pc) => CommandSource::PreClient(Arc::clone(pc)),
            InternalCommandSource::User(user_pointer) =>
            {
                // Safety: user_pointer points to data inside the object managed by `self.net`,
                // so will always survive at least as long as `self`. The returned `CommandSource`
                // creates a borrow of `self.net`, so it can't be removed while that exists.
                let user: &'_ state::User = unsafe { &**user_pointer };
                let wrapper = <wrapper::User as wrapper::ObjectWrapper>::wrap(&*self.net, user);
                CommandSource::User(wrapper)
            }
        }
    }

    fn command(&self) -> &ClientCommand
    {
        self
    }

    fn server(&self) -> &Arc<ClientServer>
    {
        &self.server
    }

    fn network(&self) -> &Arc<Network>
    {
        &self.net
    }

    fn notify_error(&self, err: CommandError)
    {
        if let Some(n) = self.translate_command_error(err)
        {
            let _ = self.response(&n.format_for(self.server(), &self.source()));
        }
    }
}

impl ClientCommand
{
    fn translate_command_error(&self, err: CommandError) -> Option<Box<dyn Numeric>>
    {
        match err
        {
            CommandError::UnderlyingError(_) => {
                todo!()
            }
            CommandError::UnknownError(_) => {
                todo!()
            }
            CommandError::CustomError => {
                todo!()
            }
            CommandError::CommandNotFound(cmd) => {
                Some(Box::new(make_numeric!(UnknownCommand, &cmd)))
            }
            CommandError::NotEnoughParameters => {
                Some(Box::new(make_numeric!(NotEnoughParameters, &self.command)))
            }
            CommandError::LookupError(le) => {
                match le
                {
                    LookupError::NoSuchNick(nick) => Some(Box::new(make_numeric!(NoSuchTarget, &nick))),
                    LookupError::NoSuchChannelName(name) => Some(Box::new(make_numeric!(NoSuchChannel, &name))),
                    _ => None
                }
            }
            CommandError::InvalidNick(name) => {
                Some(Box::new(make_numeric!(ErroneousNickname, &name)))
            }
            CommandError::InvalidChannelName(name) => {
                Some(Box::new(make_numeric!(InvalidChannelName, &name)))
            }
            CommandError::Numeric(n) => Some(n)
        }
    }
}