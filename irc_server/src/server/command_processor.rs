use irc_network::*;
use super::*;
use crate::numeric;
use crate::errors::*;
use client_listener::{
    ConnectionId,
    ConnectionError
};

use std::cell::RefCell;

/// Utility type to invoke a command handler
pub(crate) struct CommandProcessor<'a>
{
    server: &'a Server,
}

/// An action that can be triggered by a command handler.
///
/// Command handlers have only an immutable reference to the [`Server`], and so
/// cannot directly change state (with limited exceptions). If handling the command
/// requires a change in state, either network or local, then this is achieved
/// by emitting a `CommandAction` which the `Server` will apply on the next
/// iteration of its event loop.
#[derive(Debug)]
pub enum CommandAction {
    /// A network state change. The target object ID and event details are provided
    /// here; the remaining [`Event`](event::Event) fields are filled in by the
    /// event log.
    StateChange(ObjectId, event::EventDetails),
    /// Indicate that the given connection is ready to register
    RegisterClient(ConnectionId),
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
    PreClient(&'a RefCell<PreClient>),
    /// A client connection which is associated with a network user
    User(wrapper::User<'a>),
}

/// A client command to be handled
pub struct ClientCommand<'a>
{
    /// The [`Server`] instance
    pub server: &'a Server,
    /// The connection from which the command originated
    pub connection: &'a ClientConnection,
    /// Details of the user associated with the connection
    pub source: &'a CommandSource<'a>,
    /// The command being executed
    pub command: String,
    /// Arguments supplied
    pub args: Vec<String>,
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

impl ClientCommand<'_>
{
    /// Send a numeric response to the connection that invoked the command
    pub fn response(&self, n: &impl messages::Numeric) -> CommandResult
    {
        self.connection.send(&n.format_for(self.server, self.source));
        Ok(())
    }
}

impl<'a> CommandProcessor<'a>
{
    /// Construct a `CommandProcessor`
    pub fn new (server: &'a Server) -> Self
    {
        Self {
            server: server,
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
    #[tracing::instrument(skip(self))]
    pub async fn process_message(&self, message: ClientMessage)
    {
        if let Some(conn) = self.server.find_connection(message.source)
        {
            let source = self.translate_message_source(conn).expect("Got message from unknown source");
            let command = message.command.clone();

            if let Err(err) = self.do_process_message(conn, &source, message)
            {
                match err {
                    CommandError::UnderlyingError(err) => {
                        panic!("Error occurred handling command {} from {:?}: {}", command, conn.id(), err);
                    },
                    CommandError::UnknownError(desc) => {
                        panic!("Error occurred handling command {} from {:?}: {}", command, conn.id(), desc);
                    },
                    CommandError::Numeric(num) => {
                        let targeted = num.as_ref().format_for(self.server, &source);
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

    fn do_process_message(&self,
                          connection: &ClientConnection,
                          source: &CommandSource,
                          message: ClientMessage
                        ) -> Result<(), CommandError>
    {
        if let Some(factory) = self.server.command_dispatcher().resolve_command(&message.command) {
            let mut handler = factory.create(self.server, self);
            let cmd = ClientCommand {
                 server: self.server,
                 connection: connection,
                 source: source,
                 command: message.command,
                 args: message.args
            };

            handler.validate(&cmd)?;
            handler.handle(&cmd)?;
            Ok(())
        } else {
            numeric_error!(UnknownCommand, &message.command)
        }
    }

    fn translate_message_source(&self, source: &'a ClientConnection) -> Result<CommandSource<'a>, CommandError> {
        if let Some(user_id) = source.user_id {
            Ok(self.server.network().user(user_id).map(|u| CommandSource::User(u))?)
        } else if let Some(pre_client) = &source.pre_client {
            Ok(CommandSource::PreClient(pre_client))
        } else {
            Err(CommandError::unknown("Got message from neither preclient nor client"))
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
            ValidationError::InvalidUsername(e) => CommandError::UnknownError(e.0.to_string()),
            ValidationError::InvalidHostname(e) => CommandError::UnknownError(e.0.to_string()),
            ValidationError::WrongTypeId(e) => CommandError::UnknownError(e.to_string())
        }
    }
}


impl CommandError
{
    pub fn unknown(desc: impl std::fmt::Display) -> Self
    {
        Self::UnknownError(desc.to_string())
    }

    pub fn inner(err: impl std::error::Error + Clone + 'static) -> CommandError
    {
        CommandError::UnderlyingError(Box::new(err.clone()))
    }
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

impl From<policy::PermissionError> for CommandError
{
    fn from(e: policy::PermissionError) -> Self
    {
        match e
        {
            policy::PermissionError::Numeric(n) => Self::Numeric(n),
            policy::PermissionError::CustomError => Self::CustomError,
            policy::PermissionError::InternalError(e) => Self::UnderlyingError(e)
        }
    }
}