use sable_network::prelude::*;
use sable_network::network::wrapper::ObjectWrapper;
use client_listener::{
    ConnectionId,
    ConnectionError
};

use crate::*;
use crate::server::ClientServer;
use crate::messages::*;
use crate::errors::*;
use crate::capability::*;
use crate::utils::make_numeric;

use std::sync::Arc;

/// Utility type to invoke a command handler
pub(crate) struct CommandProcessor<'server>
{
    server: &'server ClientServer,
    command_dispatcher: &'server CommandDispatcher,
}

/// An action that can be triggered by a command handler.
///
/// Command handlers have only an immutable reference to the [`ClientServer`], and so
/// cannot directly change state (with limited exceptions). If handling the command
/// requires a change in state, either network or local, then this is achieved
/// by emitting a `CommandAction` which the `Server` will apply on the next
/// iteration of its event loop.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // The largest variant is also the most commonly constructed by far
pub enum CommandAction {
    /// A network state change. The target object ID and event details are provided
    /// here; the remaining [`Event`](event::Event) fields are filled in by the
    /// event log.
    StateChange(ObjectId, EventDetails),

    /// Indicate that the given connection is ready to register
    RegisterClient(ConnectionId),

    /// Attach the given connection to an existing user session
    AttachToUser(ConnectionId, UserId),

    /// Update a connection's client caps
    UpdateConnectionCaps(ConnectionId, ClientCapabilitySet),

    /// Disconnect the given user. The handler should first inform the user of the reason,
    /// if appropriate
    DisconnectUser(UserId),
}

/// An error that may occur during command processing
///
/// Note that, at present, returning the `UnderlyingError` or `UnknownError` variants
/// from a handler will cause the [`CommandProcessor`] to panic; in future this may
/// change (for example, to terminate the connection), but in either case should only
/// be used for exceptional, unhandleable, errors.
#[derive(Debug)]
pub enum CommandError
{
    /// Something returned an `Error` that we don't know how to handle
    UnderlyingError(Box<dyn std::error::Error>),
    /// Something went wrong but we don't have an `Error` impl for it
    UnknownError(String),
    /// The command couldn't be processed successfully, and the client has already been
    /// notified
    CustomError,
    /// The command couldn't be processed successfully; the provided
    /// [`Numeric`](messages::Numeric) will be sent to the client to notify them
    Numeric(Box<dyn messages::Numeric>)
}

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
pub struct ClientCommand<'server>
{
    /// The [`ClientServer`] instance
    pub server: &'server ClientServer,
    /// The connection from which the command originated
    pub connection: Arc<ClientConnection>,
    /// The network state as seen by this command handlers
    pub net: Arc<Network>,
    /// Details of the user associated with the connection
    source: InternalCommandSource,
    /// The command being executed
    pub command: String,
    /// Arguments supplied
    pub args: Vec<String>,
}

// Safety: this isn't automatically Send/Sync because of the raw pointer inside `InternalCommandSource`.
// It's safe, though, because that pointer points into an Arc<> held by the same `ClientCommand`.
unsafe impl Send for ClientCommand<'_> { }
unsafe impl Sync for ClientCommand<'_> { }

use std::slice::Iter;
use std::iter::Peekable;
use std::ops::Deref;

pub struct ArgList<'a>
{
    cmd: &'a ClientCommand<'a>,
    iter: Peekable<Iter<'a, String>>,
}

impl<'a> ArgList<'a>
{
    pub fn new(cmd: &'a ClientCommand<'a>) -> Self
    {
        Self {
            iter: cmd.args.iter().peekable(),
            cmd,
        }
    }

    pub fn next_arg(&mut self) -> Result<&String, CommandError>
    {
        Ok(self.iter.next().ok_or_else(|| make_numeric!(NotEnoughParameters, &self.cmd.command))?)
    }

    pub fn is_empty(&mut self) -> bool
    {
        self.iter.peek().is_none()
    }
/*
    pub fn iter(&mut self) -> &mut impl Iterator<Item=&'a String>
    {
        &mut self.iter
    }
*/
}

impl<'a> Deref for ArgList<'a>
{
    type Target = Peekable<Iter<'a, String>>;

    fn deref(&self) -> &Self::Target
    {
        &self.iter
    }
}

impl CommandAction
{
    /// Helper to create a [`CommandAction::StateChange`] variant. By passing the underlying
    /// ID and detail types, they will be converted into the corresponding enum variants.
    pub fn state_change(id: impl Into<ObjectId>, detail: impl Into<event::EventDetails>) -> Self
    {
        Self::StateChange(id.into(), detail.into())
    }
}

impl<'server> ClientCommand<'server>
{
    /// Construct a `ClientCommand`
    fn new(server: &'server ClientServer,
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
            args: message.args,
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

    /// Send a numeric response to the connection that invoked the command
    pub fn response(&self, n: &impl messages::Numeric) -> CommandResult
    {
        self.connection.send(&n.format_for(self.server, &self.source()));
        Ok(())
    }

    /// Return a `CommandSource` describing the originating user or connection
    pub fn source<'a>(&'a self) -> CommandSource<'a>
    {
        match &self.source
        {
            InternalCommandSource::PreClient(pc) => CommandSource::PreClient(Arc::clone(pc)),
            InternalCommandSource::User(user_pointer) =>
            {
                // Safety: user_pointer points to data inside the object managed by `self.net`,
                // so will always survive at least as long as `self`. The returned `CommandSource`
                // creates a borrow of `self.net`, so it can't be removed while that exists.
                let user: &'a state::User = unsafe { &**user_pointer };
                let wrapper = <wrapper::User as wrapper::ObjectWrapper>::wrap(&*self.net, user);
                CommandSource::User(wrapper)
            }
        }
    }
}

impl<'server> CommandProcessor<'server>
{
    /// Construct a `CommandProcessor`
    pub fn new (server: &'server ClientServer,
                command_dispatcher: &'server CommandDispatcher,
            ) -> Self
    {
        Self {
            server,
            command_dispatcher,
        }
    }

    /// Take a tokenised [`ClientMessage`] and process it as a protocol command.
    ///
    /// This function will:
    /// - Look up the source connection, identify the `PreClient` or `User`
    ///   associated with that connection
    /// - Create an appropriate [`CommandHandler`] based on the command being
    ///   executed
    /// - Invoke the handler
    /// - Process any numeric error response, if appropriate
    #[tracing::instrument(skip_all)]
    pub fn process_message<'handlers>(&self,
                                      message: ClientMessage,
                                      async_handlers: &server::AsyncHandlerCollection<'handlers>)
        where 'server: 'handlers
    {
        if let Some(conn) = self.server.find_connection(message.source)
        {
            let cmd = Arc::new(ClientCommand::new(self.server, Arc::clone(&conn), message).expect("Got message from unknown source"));

            if let Err(err) = self.do_process_message(Arc::clone(&cmd), async_handlers)
            {
                match err {
                    CommandError::UnderlyingError(err) => {
                        panic!("Error occurred handling command {} from {:?}: {}", cmd.command, conn.id(), err);
                    },
                    CommandError::UnknownError(desc) => {
                        panic!("Error occurred handling command {} from {:?}: {}", cmd.command, conn.id(), desc);
                    },
                    CommandError::Numeric(num) => {
                        let targeted = num.as_ref().format_for(self.server, &cmd.source());
                        conn.send(&targeted);
                    },
                    CommandError::CustomError => {
                    }
                }
            }
        } else {
            panic!("Got message '{}' from unknown source?", message.command);
        }
    }

    fn do_process_message<'handlers>(&self,
                          cmd: Arc<ClientCommand<'handlers>>,
                          async_handlers: &server::AsyncHandlerCollection<'handlers>,
                        ) -> Result<(), CommandError>
        where 'server: 'handlers
    {
        if let Some(factory) = self.command_dispatcher.resolve_command(&cmd.command) {
            let mut handler = factory(self.server);


            handler.validate(&cmd)?;
            if let Some(future) = handler.handle_async(Arc::clone(&cmd))
            {
                async_handlers.add(server::AsyncHandlerWrapper::new(future, cmd));
            }
            else
            {
                handler.handle(&cmd)?;
            }
            Ok(())
        } else {
            numeric_error!(UnknownCommand, &cmd.command)
        }
    }
}

impl From<ValidationError> for CommandError
{
    fn from(e: ValidationError) -> Self
    {
        match e
        {
            ValidationError::NickInUse(n) => numeric::NicknameInUse::new(&n).into(),
            ValidationError::ObjectNotFound(le) => {
                match le
                {
                    LookupError::NoSuchNick(n) | LookupError::NoSuchChannelName(n) => {
                        numeric::NoSuchTarget::new(&n).into()
                    },
                    _ => CommandError::UnknownError(le.to_string())
                }
            }
            ValidationError::InvalidNickname(e) => {
                numeric::ErroneousNickname::new(&e.0).into()
            },
            ValidationError::InvalidChannelName(e) => {
                numeric::InvalidChannelName::new(&e.0).into()
            }
            ValidationError::InvalidUsername(e) => CommandError::UnknownError(e.0),
            ValidationError::InvalidHostname(e) => CommandError::UnknownError(e.0),
            ValidationError::WrongTypeId(e) => CommandError::UnknownError(e.to_string())
        }
    }
}

impl From<policy::PermissionError> for CommandError
{
    fn from(e: policy::PermissionError) -> Self
    {
        use policy::{
            PermissionError::*,
            UserPermissionError::*,
            ChannelPermissionError::*,
        };

            match e
        {
            User(NotOper) => numeric::NotOper::new().into(),
            User(ReadOnlyUmode) => Self::CustomError, // Setting or unsetting these umodes silently fails
            Channel(channel_name, channel_err) => {
                match channel_err
                {
                    UserNotOnChannel => numeric::NotOnChannel::new(&channel_name).into(),
                    UserNotOp => numeric::ChanOpPrivsNeeded::new(&channel_name).into(),
                    UserIsBanned => numeric::BannedOnChannel::new(&channel_name).into(),
                    CannotSendToChannel => numeric::CannotSendToChannel::new(&channel_name).into(),
                    InviteOnlyChannel => numeric::InviteOnlyChannel::new(&channel_name).into(),
                    BadChannelKey => numeric::BadChannelKey::new(&channel_name).into()
                }
            },
            InternalError(e) => Self::UnderlyingError(e)
        }
    }
}

impl CommandError
{
    pub fn unknown(desc: impl std::fmt::Display) -> Self
    {
        Self::UnknownError(desc.to_string())
    }
/*
    pub fn inner(err: impl std::error::Error + Clone + 'static) -> CommandError
    {
        CommandError::UnderlyingError(Box::new(err))
    }
*/
}

impl From<LookupError> for CommandError
{
    fn from(e: LookupError) -> Self
    {
        match e {
            LookupError::NoSuchNick(n) => numeric::NoSuchTarget::new(&n).into(),
            LookupError::NoSuchChannelName(n) => numeric::NoSuchTarget::new(&n).into(),
            _ => Self::UnknownError(e.to_string())
        }
    }
}

impl From<InvalidNicknameError> for CommandError
{
    fn from(e: InvalidNicknameError) -> Self
    { numeric::ErroneousNickname::new(&e.0).into() }
}

impl From<InvalidChannelNameError> for CommandError
{
    fn from(e: InvalidChannelNameError) -> Self
    { numeric::InvalidChannelName::new(&e.0).into() }
}

impl<T: messages::Numeric + 'static> From<T> for CommandError
{
    fn from(t: T) -> Self {
        Self::Numeric(Box::new(t))
    }
}

impl From<ConnectionError> for CommandError
{
    fn from(e: ConnectionError) -> Self {
        Self::UnderlyingError(Box::new(e))
    }
}

impl From<HandlerError> for CommandError
{
    fn from(e: HandlerError) -> Self {
        Self::UnderlyingError(Box::new(e))
    }
}
