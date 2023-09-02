use sable_network::audit::AuditLogger;

use super::*;

use std::str::FromStr;

/// Trait to be implemented for any type that can be an ambient parameter (i.e. one that does not
/// originate from a positional command parameter, but is taken from the command context) to a command
/// handler function
pub trait AmbientArgument<'a>: Sized + Send + Sync
where
    Self: 'a,
{
    /// Attempt to extract an argument of this type from the provided command context and argument list.
    /// The entry point into this trait.
    ///
    /// Callers should check for an `Err` return and notify the originator of the command that an error
    /// was encountered.
    fn load_from(ctx: &'a dyn Command) -> Result<Self, CommandError>;
}

/// Trait to be implemented for any type that can be a positional parameter to a command handler function
pub trait PositionalArgument<'a>: Sized + Send + Sync
where
    Self: 'a,
{
    /// Attempt to extract an argument of this type from the provided command context and argument list.
    /// The entry point into this trait. The default implementation attempts to extract a string value
    /// from `arg_list` and passes it to [`parse_str`](Self::parse_str).
    ///
    /// Callers should check for an `Err` return and notify the originator of the command that an error
    /// was encountered.
    fn parse<'b>(
        ctx: &'a dyn Command,
        arg_list: &'b mut ArgListIter<'a>,
    ) -> Result<Self, CommandError>
    where
        'a: 'b,
    {
        let s = arg_list.next().ok_or(CommandError::NotEnoughParameters)?;
        Self::parse_str(ctx, s)
    }

    /// Parse an argument of this type from the given string value. This is called by the default
    /// implementation of [`parse`](Self::parse).
    fn parse_str(ctx: &'a dyn Command, value: &'a str) -> Result<Self, CommandError>;
}

impl<'a> PositionalArgument<'a> for Nickname {
    fn parse_str(_ctx: &'a dyn Command, value: &'a str) -> Result<Self, CommandError> {
        Ok(Nickname::from_str(value)?)
    }
}

impl<'a> PositionalArgument<'a> for Username {
    fn parse_str(_ctx: &'a dyn Command, value: &'a str) -> Result<Self, CommandError> {
        Ok(Username::new_coerce(value)?)
    }
}

impl<'a> PositionalArgument<'a> for state::ChannelRoleName {
    fn parse_str(_ctx: &'a dyn Command, value: &'a str) -> Result<Self, CommandError> {
        value
            .parse()
            .map_err(|_| CommandError::InvalidArgument(value.to_string(), "role name".to_string()))
    }
}

impl<'a> PositionalArgument<'a> for CustomRoleName {
    fn parse_str(_ctx: &'a dyn Command, value: &'a str) -> Result<Self, CommandError> {
        value.parse().map_err(|_| {
            CommandError::InvalidArgument(value.to_string(), "custom role name".to_string())
        })
    }
}

impl<'a> PositionalArgument<'a> for wrapper::User<'a> {
    fn parse_str(ctx: &'a dyn Command, s: &'a str) -> Result<Self, CommandError> {
        Ok(ctx.network().user_by_nick(&Nickname::from_str(s)?)?)
    }
}

impl<'a> PositionalArgument<'a> for wrapper::Account<'a> {
    fn parse_str(ctx: &'a dyn Command, value: &'a str) -> Result<Self, CommandError> {
        Ok(ctx.network().account_by_name(&Nickname::from_str(value)?)?)
    }
}

impl<'a> PositionalArgument<'a> for wrapper::Channel<'a> {
    fn parse_str(ctx: &'a dyn Command, s: &'a str) -> Result<Self, CommandError> {
        Ok(ctx.network().channel_by_name(&ChannelName::from_str(s)?)?)
    }
}

impl<'a> PositionalArgument<'a> for wrapper::ChannelRegistration<'a> {
    fn parse_str(ctx: &'a dyn Command, s: &'a str) -> Result<Self, CommandError> {
        Ok(ctx
            .network()
            .channel_registration_by_name(ChannelName::from_str(s)?)?)
    }
}

impl<'a> AmbientArgument<'a> for &'a dyn Command {
    fn load_from(ctx: &'a dyn Command) -> Result<Self, CommandError> {
        Ok(ctx)
    }
}

impl<'a> AmbientArgument<'a> for &'a dyn CommandResponse {
    fn load_from(ctx: &'a dyn Command) -> Result<Self, CommandError> {
        Ok(ctx.response_sink())
    }
}

impl<'a> AmbientArgument<'a> for &'a ClientServer {
    fn load_from(ctx: &'a dyn Command) -> Result<Self, CommandError> {
        Ok(ctx.server())
    }
}

impl<'a> AmbientArgument<'a> for &'a Network {
    fn load_from(ctx: &'a dyn Command) -> Result<Self, CommandError> {
        Ok(ctx.network().as_ref())
    }
}

impl<'a> AmbientArgument<'a> for AuditLogger<'a> {
    fn load_from(ctx: &'a dyn Command) -> Result<Self, CommandError> {
        Ok(AuditLogger::new(
            ctx.server().node(),
            ctx.source().user().map(|u| u.id()),
            ctx.connection().remote_addr(),
            ctx.command().to_string(),
        ))
    }
}

impl<'a> PositionalArgument<'a> for &'a str {
    fn parse_str(_ctx: &'a dyn Command, s: &'a str) -> Result<Self, CommandError> {
        Ok(s)
    }
}

impl<'a> PositionalArgument<'a> for u32 {
    fn parse_str(_ctx: &'a dyn Command, value: &'a str) -> Result<Self, CommandError> {
        value
            .parse()
            .map_err(|_| CommandError::UnknownError("failed to parse integer argument".to_owned()))
    }
}

impl<'a, T: PositionalArgument<'a>> PositionalArgument<'a> for Option<T> {
    fn parse<'b>(
        ctx: &'a dyn Command,
        arg_list: &'b mut ArgListIter<'a>,
    ) -> Result<Self, CommandError>
    where
        'a: 'b,
    {
        Ok(T::parse(ctx, arg_list).ok())
    }

    fn parse_str(_ctx: &'a dyn Command, _value: &'a str) -> Result<Self, CommandError> {
        unreachable!();
    }
}

/// Either Ok(parsed_value) or Err(raw_value) if it cannot be parsed.
/// Never actually returns a CommandError, because the correct reply varies
/// (NotEnoughParameters for USER, 'FAIL SETNAME INVALID_REALNAME' for SETNAME)
impl<'a> PositionalArgument<'a> for Result<Realname, &'a str> {
    fn parse<'b>(
        _ctx: &'a dyn Command,
        arg_list: &'b mut ArgListIter<'a>,
    ) -> Result<Self, CommandError>
    where
        'a: 'b,
    {
        let s = arg_list.next().ok_or(CommandError::NotEnoughParameters)?;
        Ok(Realname::new_coerce(s).map_err(|_| s))
    }

    fn parse_str(_ctx: &'a dyn Command, _value: &'a str) -> Result<Self, CommandError> {
        unreachable!();
    }
}

/// Either Ok(parsed_value) or Err(raw_value) if it cannot be parsed.
/// Never actually returns a CommandError.
impl<'a, T: PositionalArgument<'a>> PositionalArgument<'a> for Result<T, &'a str> {
    fn parse<'b>(
        ctx: &'a dyn Command,
        arg_list: &'b mut ArgListIter<'a>,
    ) -> Result<Self, CommandError>
    where
        'a: 'b,
    {
        let s = arg_list.next().ok_or(CommandError::NotEnoughParameters)?;
        Ok(T::parse_str(ctx, s).map_err(|_| s))
    }

    fn parse_str(_ctx: &'a dyn Command, _value: &'a str) -> Result<Self, CommandError> {
        unreachable!();
    }
}
