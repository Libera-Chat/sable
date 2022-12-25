use super::*;

use utils::ClientCommandExt;

/// CommandContext wrapper which replaces the error notification code to send
/// notices instead of numerics.
pub struct ServicesCommand<'a>
{
    outer: &'a dyn Command,
    command: &'a str,
    args: ArgListIter<'a>,
}

impl<'a> ServicesCommand<'a>
{
    pub fn new(outer: &'a dyn Command, command: &'a str, args: ArgListIter<'a>) -> Self
    {
        Self {
            outer,
            command,
            args
        }
    }
}

impl<'a> Command for ServicesCommand<'a>
{
    fn source(&self) -> CommandSource<'_> { self.outer.source() }
    fn server(&self) -> &Arc<ClientServer> { self.outer.server() }
    fn network(&self) -> &Arc<Network> { self.outer.network() }
    fn response(&self, message: &dyn messages::MessageTypeFormat) { self.outer.response(message) }
    fn connection(&self) -> client_listener::ConnectionId { self.outer.connection() }

    fn command(&self) -> &str
    {
        self.command
    }

    fn args(&self) -> ArgListIter
    {
        self.args.clone()
    }

    fn notify_error(&self, err: CommandError)
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
                self.notice(format_args!("Unknown command {}", cmd.to_ascii_uppercase()));
            }
            CommandError::NotEnoughParameters => {
                self.notice("Invalid parameters");
            }
            CommandError::LookupError(le) => {
                match le
                {
                    LookupError::NoSuchNick(nick) => {
                        self.notice(format_args!("There is no such nick {}", nick))
                    }
                    LookupError::NoSuchChannelName(name) => {
                        self.notice(format_args!("Channel {} does not exist", name))
                    }
                    LookupError::NoSuchAccountNamed(name) => {
                        self.notice(format_args!("{} is not registered", name))
                    }
                    LookupError::ChannelNotRegistered(name) => {
                        self.notice(format_args!("{} is not registered", name))
                    }
                    err => {
                        self.notice(format_args!("Unknown error: {}", err))
                    }
                }
            }
            CommandError::InvalidNick(name) => {
                self.notice(format_args!("Invalid nickname {}", name));
            }
            CommandError::InvalidChannelName(name) => {
                self.notice(format_args!("Invalid channel name {}", name));
            }
            CommandError::ServicesNotAvailable => {
                self.notice("Services are currently unavailable");
            }
            CommandError::NotLoggedIn => {
                self.notice("You are not logged in");
            }
            CommandError::ChannelNotRegistered(c) => {
                self.notice(format_args!("Channel {} is not registered", c));
            }
            CommandError::InvalidArgument(arg, ty) => {
                self.notice(format_args!("{} is not a valid {}", arg, ty));
            }
            CommandError::Numeric(n) => {
                tracing::warn!("Translating unknown error numeric from services response: {:?}", n);
                self.notice(format_args!("Unknown error: {}", n.message()));
            }
        }
    }
}


mod ns;
mod cs;