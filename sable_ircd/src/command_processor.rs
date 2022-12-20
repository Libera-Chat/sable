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
    server: Arc<ClientServer>,
    command_dispatcher: &'server CommandDispatcher,
}



use std::slice::Iter;
use std::iter::Peekable;
use std::ops::Deref;

pub struct ArgList<'a>
{
    cmd: &'a ClientCommand,
    iter: Peekable<Iter<'a, String>>,
}

impl<'a> ArgList<'a>
{
    pub fn new(cmd: &'a ClientCommand) -> Self
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

impl<'server> CommandProcessor<'server>
{
    /// Construct a `CommandProcessor`
    pub fn new (server: Arc<ClientServer>,
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
            let cmd = Arc::new(ClientCommand::new(Arc::clone(&self.server),
                                                  Arc::clone(&conn),
                                                  message
                                            ).expect("Got message from unknown source"));

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
                        let targeted = num.as_ref().format_for(&self.server, &cmd.source());
                        conn.send(&targeted);
                    },
                    _ => {
                    }
                }
            }
        } else {
            panic!("Got message '{}' from unknown source?", message.command);
        }
    }

    fn do_process_message<'handlers>(&self,
                          cmd: Arc<ClientCommand>,
                          async_handlers: &server::AsyncHandlerCollection<'handlers>,
                        ) -> Result<(), CommandError>
        where 'server: 'handlers
    {
        if let Some(factory) = self.command_dispatcher.resolve_command(&cmd.command) {
            let mut handler = factory(Arc::clone(&self.server));


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
