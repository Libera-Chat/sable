use super::*;
use sable_network::policy::*;

/// CommandContext wrapper which replaces the error notification code to send
/// notices instead of numerics.
pub struct ServicesCommand<'a> {
    outer: &'a dyn Command,
    command: &'a str,
    args: ArgListIter<'a>,
    is_from_alias: Option<&'a wrapper::User<'a>>,
}

impl<'a> ServicesCommand<'a> {
    pub fn new(
        outer: &'a dyn Command,
        command: &'a str,
        args: ArgListIter<'a>,
        is_from_alias: Option<&'a wrapper::User<'a>>,
    ) -> Self {
        Self {
            outer,
            command,
            args,
            is_from_alias,
        }
    }
}

impl Command for ServicesCommand<'_> {
    fn source(&self) -> CommandSource<'_> {
        self.outer.source()
    }
    fn server(&self) -> &Arc<ClientServer> {
        self.outer.server()
    }
    fn network(&self) -> &Arc<Network> {
        self.outer.network()
    }
    fn response_sink(&self) -> &dyn CommandResponse {
        self.outer.response_sink()
    }
    fn response_sink_arc(&self) -> Arc<dyn CommandResponse + 'static> {
        self.outer.response_sink_arc()
    }
    fn connection_id(&self) -> client_listener::ConnectionId {
        self.outer.connection_id()
    }
    fn connection(&self) -> &ClientConnection {
        self.outer.connection()
    }

    fn response_source(&self) -> &dyn messages::MessageSource {
        if let Some(alias) = self.is_from_alias {
            alias
        } else {
            self.server()
        }
    }

    fn command(&self) -> &str {
        self.command
    }

    fn args(&self) -> ArgListIter<'_> {
        self.args.clone()
    }

    fn notify_error(&self, err: CommandError) {
        match err {
            CommandError::UnderlyingError(_) => {
                todo!()
            }
            CommandError::UnknownError(_) => {
                todo!()
            }
            /*
            CommandError::CustomError => {
                todo!()
            }
            */
            CommandError::CommandNotFound(cmd) => {
                self.notice(format_args!("Unknown command {}", cmd.to_ascii_uppercase()));
            }
            CommandError::NotEnoughParameters => {
                self.notice("Invalid parameters");
            }
            CommandError::LookupError(le) => match le {
                LookupError::NoSuchNick(nick) => {
                    self.notice(format_args!("There is no such nick {nick}"))
                }
                LookupError::NoSuchChannelName(name) => {
                    self.notice(format_args!("Channel {name} does not exist"))
                }
                LookupError::NoSuchAccountNamed(name) => {
                    self.notice(format_args!("{name} is not registered"))
                }
                LookupError::ChannelNotRegistered(name) => {
                    self.notice(format_args!("{name} is not registered"))
                }
                err => self.notice(format_args!("Unknown error: {err}")),
            },
            CommandError::InvalidNickname(name) => {
                self.notice(format_args!("Invalid nickname {name}"));
            }
            CommandError::InvalidUsername(name) => {
                self.notice(format_args!("Invalid username {name}"));
            }
            CommandError::InvalidChannelName(name) => {
                self.notice(format_args!("Invalid channel name {name}"));
            }
            CommandError::ServicesNotAvailable => {
                self.notice("Services are currently unavailable");
            }
            CommandError::NotLoggedIn => {
                self.notice("You are not logged in");
            }
            CommandError::ChannelNotRegistered(c) => {
                self.notice(format_args!("Channel {c} is not registered"));
            }
            CommandError::InvalidArgument(arg, ty) => {
                self.notice(format_args!("{arg} is not a valid {ty}"));
            }
            CommandError::Permission(pe) => {
                match pe {
                    PermissionError::User(ue) =>
                    {
                        use UserPermissionError::*;
                        #[allow(clippy::single_match)] // For consistency with other branches
                        match ue {
                            NotLoggedIn => { self.notice("You are not logged in") }
                            _ => {}
                        }
                    }
                    PermissionError::Channel(chan, ce) =>
                    {
                        use ChannelPermissionError::*;
                        match ce {
                            NotRegistered => { self.notice(format_args!("{chan} is not registered")) },
                            NoAccess => { self.notice("Access denied") }
                            _ => {}
                        }
                    }
                    PermissionError::Registration(re) =>
                    {
                        match re
                        {
                            RegistrationPermissionError::NotLoggedIn => { self.notice("You are not logged in") }
                            RegistrationPermissionError::NoAccess => { self.notice("Access denied") }
                            RegistrationPermissionError::CantEditHigherRole => { self.notice("Access denied - you can't edit a role with permissions you don't have yourself") }
                        }
                    }
                    PermissionError::InternalError(_) => todo!(),
                }
            }
            CommandError::Numeric(n) => {
                tracing::warn!(
                    "Translating unknown error numeric from services response: {:?}",
                    n
                );
                self.notice(format_args!("Unknown error: {}", n.debug_format()));
            }
            CommandError::Fail {
                command,
                code,
                context,
                description,
            } => {
                tracing::warn!(
                    "Translating unknown error numeric from services response: {} {} {} :{}",
                    command,
                    code,
                    context,
                    description
                );
                self.notice(format_args!(
                    "Unknown error: {command} {code} {context} :{description}"
                ));
            }
        }
    }
}

mod dispatch_alias;
pub use dispatch_alias::*;

mod cs;
mod ns;
mod sasl;
