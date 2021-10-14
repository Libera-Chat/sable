use crate::ircd::*;
use super::*;
use crate::utils::*;
use std::cell::RefCell;

pub struct CommandProcessor<'a>
{
    server: &'a Server,
    actions: Vec<CommandAction>
}

pub enum CommandAction {
    StateChange(event::Event),
    RegisterClient(ConnectionId),
}

pub enum CommandError
{
    UnderlyingError(Box<dyn std::error::Error>),
    UnknownError(String),
    InvalidCommand,
    NotEnoughParameters,
    NotRegistered,
    AlreadyRegistered,
    NoSuchTarget(String),
}

pub enum CommandSource<'a>
{
    PreClient(&'a RefCell<PreClient>),
    User(wrapper::User<'a>),
}

pub struct ClientCommand<'a>
{
    pub connection: &'a ClientConnection,
    pub source: CommandSource<'a>,
    pub command: String,
    pub args: Vec<String>,
}

impl<'a> CommandProcessor<'a>
{
    pub fn new (server: &'a Server) -> Self
    {
        Self {
            server: server,
            actions: Vec::new()
        }
    }

    pub fn actions(self) -> Vec<CommandAction>
    {
        self.actions
    }

    pub async fn process_message(&mut self, message: ClientMessage)
    {
        if let Some(conn) = self.server.find_connection(message.source)
        {
            let command = message.command.clone();
            if let Err(err) = self.do_process_message(conn, message)
            {
                match err {
                    CommandError::UnderlyingError(err) => {
                        panic!("Error occurred handling command {} from {:?}: {}", command, conn.id(), err);
                    },
                    CommandError::UnknownError(desc) => {
                        panic!("Error occurred handling command {} from {:?}: {}", command, conn.id(), desc);
                    }
                    CommandError::InvalidCommand => {
                        conn.connection.send(&format!(":{} 421 * {} :Unknown command\r\n", self.server.name(), command)).await
                                .or_log("sending error numeric");
                    },
                    CommandError::NotEnoughParameters => {
                        conn.connection.send(&format!(":{} 461 * {} :Not enough parameters\r\n", self.server.name(), command)).await
                        .or_log("sending error numeric");
                    },
                    CommandError::NotRegistered => {
                        conn.connection.send(&format!(":{} 451 * :You have not registered\r\n", self.server.name())).await
                        .or_log("sending error numeric");
                    },
                    CommandError::AlreadyRegistered => {
                        conn.connection.send(&format!(":{} 462 * :You are already connected and cannot handshake again\r\n", self.server.name())).await
                        .or_log("sending error numeric");
                    },
                    CommandError::NoSuchTarget(target) => {
                        conn.connection.send(&format!(":{} 401 * {} :No such nick/channel", self.server.name(), target)).await
                        .or_log("sending error numeric");
                    }
                }
            }
        } else {
            panic!("Got message '{}' from unknown source?", message.command);
        }
    }

    fn do_process_message(&mut self, connection: &ClientConnection, message: ClientMessage) -> Result<(), CommandError>
    {
        if let Some(handler) = self.server.command_dispatcher().resolve_command(&message.command) {
            let cmd = ClientCommand {
                 connection: connection, 
                 source: self.translate_message_source(connection)?,
                 command: message.command,
                 args: message.args
            };

            handler.validate(self.server, &cmd)?;
            handler.handle(self.server, &cmd, &mut self.actions)
        } else {
            Err(CommandError::InvalidCommand)
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
            LookupError::NoSuchNick(n) => Self::NoSuchTarget(n),
            LookupError::NoSuchChannelName(n) => Self::NoSuchTarget(n),
            _ => Self::UnknownError(e.to_string())
        }
    }
}