use irc_network::*;
use super::*;
use crate::numeric;
use crate::errors::*;

use std::cell::RefCell;

pub struct CommandProcessor<'a>
{
    server: &'a Server,
}

#[derive(Debug)]
pub enum CommandAction {
    StateChange(ObjectId, event::EventDetails),
    RegisterClient(ConnectionId),
    DisconnectUser(UserId),
}

#[derive(Debug)]
pub enum CommandError
{
    UnderlyingError(Box<dyn std::error::Error>),
    CustomError,
    UnknownError(String),
    Numeric(Box<dyn messages::Numeric>)
}

pub enum CommandSource<'a>
{
    PreClient(&'a RefCell<PreClient>),
    User(wrapper::User<'a>),
}

pub struct ClientCommand<'a>
{
    pub server: &'a Server,
    pub connection: &'a ClientConnection,
    pub source: &'a CommandSource<'a>,
    pub command: String,
    pub args: Vec<String>,
}

impl CommandAction
{
    pub fn state_change(id: impl Into<ObjectId>, detail: impl Into<event::EventDetails>) -> Self
    {
        Self::StateChange(id.into(), detail.into())
    }
}

impl ClientCommand<'_>
{
    pub fn response(&self, n: &impl messages::Numeric) -> CommandResult
    {
        self.connection.send(&n.format_for(self.server, self.source));
        Ok(())
    }
}

impl<'a> CommandProcessor<'a>
{
    pub fn new (server: &'a Server) -> Self
    {
        Self {
            server: server,
        }
    }

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