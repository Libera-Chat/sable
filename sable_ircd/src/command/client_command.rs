use crate::capability::ClientCapability;

use super::{
    plumbing::{Command, CommandExt, CommandResponse, LabeledResponseSink, PlainResponseSink},
    *,
};
use sable_network::{network::wrapper::ObjectWrapper, policy::*};

/// Describes the possible types of connection that can invoke a command handler
pub enum CommandSource<'a> {
    /// A client connection which has not yet completed registration
    PreClient(Arc<PreClient>),
    /// A client connection which is associated with a network user
    User(wrapper::User<'a>, wrapper::UserConnection<'a>),
}

impl<'a> CommandSource<'a> {
    pub fn user(&self) -> Option<&wrapper::User<'a>> {
        match self {
            Self::PreClient(_) => None,
            Self::User(u, _) => Some(u),
        }
    }

    pub fn pre_client(&self) -> Option<&PreClient> {
        match self {
            Self::PreClient(pc) => Some(pc.as_ref()),
            Self::User(_, _) => None,
        }
    }
}

impl std::fmt::Debug for CommandSource<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PreClient(arg0) => f.debug_tuple("PreClient").field(arg0).finish(),
            Self::User(arg0, arg1) => f
                .debug_tuple("User")
                .field(&format!("id={:?}", arg0.id()))
                .field(&format!("nick={}", arg0.nick()))
                .field(&format!("conn_id={:?}", arg1.id()))
                .finish(),
        }
    }
}

/// Internal representation of a `CommandSource`
enum InternalCommandSource {
    PreClient(Arc<PreClient>),
    User(*const state::User, *const state::UserConnection),
}

/// A client command to be handled
pub struct ClientCommand {
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
    pub args: Vec<String>,
    /// Tags provided by the client
    #[expect(unused)]
    pub tags: InboundTagSet,

    // The response sink. labeled-response requires that this lives for the whole
    // lifetime of the command, not just the handler duration, because even translated
    // error returns need to be inside the single batch
    response_sink: Arc<dyn CommandResponse + 'static>,
}

// Safety: this isn't automatically Send/Sync because of the raw pointer inside `InternalCommandSource`.
// It's safe, though, because that pointer points into an Arc<> held by the same `ClientCommand`.
unsafe impl Send for ClientCommand {}
unsafe impl Sync for ClientCommand {}

impl ClientCommand {
    /// Construct a `ClientCommand`
    pub fn new(
        server: Arc<ClientServer>,
        connection: Arc<ClientConnection>,
        message: ClientMessage,
    ) -> Result<Self, CommandError> {
        let net = server.network();
        let source = Self::translate_message_source(&net, &connection)?;
        let response_target = Self::translate_internal_source(&source, net.as_ref()).format();
        let response_sink = Self::make_response_sink(
            Arc::clone(&connection),
            &message.tags,
            server.format(),
            response_target,
        );

        Ok(Self {
            server,
            connection,
            net,
            source,
            command: message.command,
            args: message.args,
            tags: message.tags,
            response_sink,
        })
    }

    // Create the appropriate internal response sink
    fn make_response_sink(
        conn: Arc<ClientConnection>,
        inbound_tags: &InboundTagSet,
        response_source: String,
        response_target: String,
    ) -> Arc<dyn CommandResponse + 'static> {
        if conn.capabilities.has(ClientCapability::LabeledResponse) {
            if let Some(label) = inbound_tags.has("label") {
                if let Some(label) = &label.value {
                    return Arc::new(LabeledResponseSink::new(
                        response_source,
                        response_target,
                        conn,
                        label.clone(),
                    ));
                }
            }
        }
        Arc::new(PlainResponseSink::new(
            response_source,
            response_target,
            conn,
        ))
    }

    fn translate_message_source(
        net: &Network,
        source: &ClientConnection,
    ) -> Result<InternalCommandSource, CommandError> {
        if let Some((user_id, conn_id)) = source.user_ids() {
            let user_state = net.user(user_id)?.raw();
            let conn_state = net.user_connection(conn_id)?.raw();
            Ok(InternalCommandSource::User(user_state, conn_state))
        } else if let Some(pre_client) = source.pre_client() {
            Ok(InternalCommandSource::PreClient(pre_client))
        } else {
            Err(CommandError::unknown(
                "Got message from neither preclient nor client",
            ))
        }
    }

    fn translate_internal_source<'a>(
        source: &'a InternalCommandSource,
        net: &'a Network,
    ) -> CommandSource<'a> {
        match source {
            InternalCommandSource::PreClient(pc) => CommandSource::PreClient(Arc::clone(pc)),
            InternalCommandSource::User(user_pointer, conn_pointer) => {
                // Safety: user_pointer points to data inside the object managed by `self.net`,
                // so will always survive at least as long as `self`. The returned `CommandSource`
                // creates a borrow of `self.net`, so it can't be removed while that exists.
                let user: &'_ state::User = unsafe { &**user_pointer };
                let user_conn: &'_ state::UserConnection = unsafe { &**conn_pointer };
                let user_wrapper = <wrapper::User as wrapper::ObjectWrapper>::wrap(net, user);
                let user_conn_wrapper =
                    <wrapper::UserConnection as wrapper::ObjectWrapper>::wrap(net, user_conn);
                CommandSource::User(user_wrapper, user_conn_wrapper)
            }
        }
    }
}

impl Command for ClientCommand {
    fn source(&self) -> CommandSource<'_> {
        Self::translate_internal_source(&self.source, self.net.as_ref())
    }

    fn command(&self) -> &str {
        &self.command
    }

    fn args(&self) -> ArgListIter<'_> {
        ArgListIter::new(&self.args)
    }

    fn server(&self) -> &Arc<ClientServer> {
        &self.server
    }

    fn network(&self) -> &Arc<Network> {
        &self.net
    }

    fn notify_error(&self, err: CommandError) {
        self.send_command_error(err)
    }

    fn response_sink(&self) -> &dyn CommandResponse {
        self.response_sink.as_ref()
    }

    fn response_sink_arc(&self) -> Arc<dyn CommandResponse + 'static> {
        self.response_sink.clone()
    }

    fn connection_id(&self) -> client_listener::ConnectionId {
        self.connection.id()
    }

    fn connection(&self) -> &ClientConnection {
        self.connection.as_ref()
    }

    fn response_source(&self) -> &dyn messages::MessageSource {
        self.server()
    }
}

impl ClientCommand {
    fn send_command_error(&self, err: CommandError) {
        let numeric = match err {
            CommandError::UnderlyingError(_) => {
                tracing::error!(?self.command, ?self.args, source = ?self.source(), "Got unhandled error: {:?}", err);
                None
            }
            CommandError::UnknownError(_) => {
                todo!()
            }
            /*
                        CommandError::CustomError => {
                            todo!()
                        }
            */
            CommandError::CommandNotFound(cmd) => Some(make_numeric!(UnknownCommand, &cmd)),
            CommandError::NotEnoughParameters => {
                Some(make_numeric!(NotEnoughParameters, &self.command))
            }
            CommandError::LookupError(le) => match le {
                LookupError::NoSuchNick(nick) => Some(make_numeric!(NoSuchTarget, &nick)),
                LookupError::NoSuchChannelName(name) => Some(make_numeric!(NoSuchChannel, &name)),
                _ => None,
            },
            CommandError::InvalidNickname(name) => Some(make_numeric!(ErroneousNickname, &name)),
            CommandError::InvalidUsername(_name) => Some(make_numeric!(InvalidUsername)),
            CommandError::InvalidChannelName(name) => {
                Some(make_numeric!(InvalidChannelName, &name))
            }
            CommandError::ServicesNotAvailable => Some(make_numeric!(ServicesNotAvailable)),
            CommandError::NotLoggedIn
            | CommandError::Permission(PermissionError::User(UserPermissionError::NotLoggedIn))
            | CommandError::Permission(PermissionError::Registration(
                RegistrationPermissionError::NotLoggedIn,
            )) => {
                self.notice("You are not logged in");
                None
            }
            CommandError::ChannelNotRegistered(c) => {
                self.notice(format_args!("Channel {c} is not registered"));
                None
            }
            CommandError::InvalidArgument(arg, ty) => {
                self.notice(format_args!("{arg} is not a valid {ty}"));
                None
            }
            CommandError::Permission(pe) => {
                match pe {
                    // These have no corresponding numerics
                    PermissionError::User(_) => None,
                    PermissionError::Registration(_) => None,
                    // These ones we can translate
                    PermissionError::Channel(channel_name, channel_err) => {
                        use ChannelPermissionError::*;

                        match channel_err {
                            NotOnChannel => Some(make_numeric!(NotOnChannel, &channel_name)),
                            UserNotOp => Some(make_numeric!(ChanOpPrivsNeeded, &channel_name)),
                            UserIsBanned => Some(make_numeric!(BannedOnChannel, &channel_name)),
                            CannotSendToChannel => {
                                Some(make_numeric!(CannotSendToChannel, &channel_name))
                            }
                            InviteOnlyChannel => {
                                Some(make_numeric!(InviteOnlyChannel, &channel_name))
                            }
                            BadChannelKey => Some(make_numeric!(BadChannelKey, &channel_name)),
                            NotRegistered | NoAccess => None,
                        }
                    }
                    PermissionError::InternalError(_) => None,
                }
            }
            CommandError::Numeric(n) => Some(n),
            CommandError::Fail {
                command,
                code,
                context,
                description,
            } => {
                self.response_sink
                    .send(message::Fail::new(command, code, &context, &description));
                None
            }
        };
        if let Some(numeric) = numeric {
            self.response_sink.numeric(numeric)
        }
    }
}
