use super::*;

use std::str::FromStr;

/// Used as an associated type param for [`ArgumentType`] to define a positional argument type
pub struct PositionalArgumentType<T>(T);
/// Used as an associated type param for [`ArgumentType`] to define an ambient argument type,
/// i.e. one that doesn't consume a positional argument
pub struct AmbientArgumentType<T>(T);
/// Used as an associated type param for [`ArgumentType`] to define a custom argument type,
/// which may or may not consume a positional argument
pub struct CustomArgumentType<T>(T);

mod private {
    use super::*;

    /// Private trait used to ensure that [`PositionalArgumentType`], [`AmbientArgumentType`] and
    /// [`CustomArgumentType`] are the only valid possibilities for `ArgumentType::Category`.
    pub trait ArgumentTypeCategory<'a, T>
    {
        fn parse(ctx: &'a impl CommandContext, arg: &mut ArgumentListIter<'a>) -> Result<T, CommandError>;
    }

    impl<'a, T> ArgumentTypeCategory<'a, T> for super::PositionalArgumentType<T>
        where T: ArgumentType<'a, Category = super::PositionalArgumentType<T>>
    {
        fn parse(ctx: &'a impl CommandContext, arg: &mut ArgumentListIter<'a>) -> Result<T, CommandError>
        {
            let s = arg.next().ok_or(CommandError::NotEnoughParameters)?;
            T::parse_str(ctx, s)
        }
    }
    impl<'a, T> ArgumentTypeCategory<'a, T> for super::AmbientArgumentType<T>
        where T: ArgumentType<'a, Category = super::AmbientArgumentType<T>>
    {
        fn parse(ctx: &'a impl CommandContext, _arg: &mut ArgumentListIter<'a>) -> Result<T, CommandError>
        {
            T::parse_ambient(ctx)
        }
    }
    impl<'a, T> ArgumentTypeCategory<'a, T> for super::CustomArgumentType<T>
        where T: ArgumentType<'a, Category = super::CustomArgumentType<T>>
    {
        fn parse(ctx: &'a impl CommandContext, arg: &mut ArgumentListIter<'a>) -> Result<T, CommandError>
        {
            T::parse_custom(ctx, arg)
        }
    }
}

/// Trait to be implemented for any type that can be a parameter to a command handler function
pub trait ArgumentType<'a> : Sized + Send + Sync
    where Self: 'a
{
    type Category: private::ArgumentTypeCategory<'a, Self>;

    /// Attempt to extract an argument of this type from the provided command context and argument list.
    /// The entry point into this trait.
    ///
    /// Callers should check for an `Err` return and notify the originator of the command that an error
    /// was encountered.
    fn parse(ctx: &'a impl CommandContext, arg: &mut ArgumentListIter<'a>) -> Result<Self, CommandError>
    {
        use private::ArgumentTypeCategory;
        Self::Category::parse(ctx, arg)
    }

    /// For positional argument types, extract a value of this type from the given string argument
    fn parse_str(_ctx: &'a impl CommandContext, _value: &'a str) -> Result<Self, CommandError>
        where Self: ArgumentType<'a, Category = PositionalArgumentType<Self>>
    { unimplemented!(); }

    /// For ambient argument types, extract a value from the provided context
    fn parse_ambient(_ctx: &'a impl CommandContext) -> Result<Self, CommandError>
        where Self: ArgumentType<'a, Category = AmbientArgumentType<Self>>
    { unimplemented!(); }

    /// For custom argument types, extract a value however is necessary
    fn parse_custom(_ctx: &'a impl CommandContext, _arg: &mut ArgumentListIter<'a>) -> Result<Self, CommandError>
        where Self: ArgumentType<'a, Category = CustomArgumentType<Self>>
    { unimplemented!(); }
}

impl<'a> ArgumentType<'a> for Nickname
{
    type Category = PositionalArgumentType<Self>;
    fn parse_str(_ctx: &'a impl CommandContext, value: &'a str) -> Result<Self, CommandError>
    {
        Ok(Nickname::from_str(value)?)
    }
}

impl<'a> ArgumentType<'a> for ChannelKey
{
    type Category = PositionalArgumentType<Self>;
    fn parse_str(_ctx: &'a impl CommandContext, value: &'a str) -> Result<Self, CommandError>
    {
        Ok(ChannelKey::new_coerce(value))
    }
}

impl<'a> ArgumentType<'a> for wrapper::User<'a>
{
    type Category = PositionalArgumentType<Self>;
    fn parse_str(ctx: &'a impl CommandContext, s: &'a str) -> Result<Self, CommandError>
    {
        Ok(ctx.network().user_by_nick(&Nickname::from_str(s)?)?)
    }
}

impl<'a> ArgumentType<'a> for wrapper::Channel<'a>
{
    type Category = PositionalArgumentType<Self>;
    fn parse_str(ctx: &'a impl CommandContext, s: &'a str) -> Result<Self, CommandError>
    {
        Ok(ctx.network().channel_by_name(&ChannelName::from_str(s)?)?)
    }
}

impl<'a> ArgumentType<'a> for &'a ClientCommand
{
    type Category = AmbientArgumentType<Self>;
    fn parse_ambient(ctx: &'a impl CommandContext) -> Result<Self, CommandError>
    {
        Ok(ctx.command())
    }
}

impl<'a> ArgumentType<'a> for &'a ClientServer
{
    type Category = AmbientArgumentType<Self>;
    fn parse_ambient(ctx: &'a impl CommandContext) -> Result<Self, CommandError>
    {
        Ok(ctx.server())
    }
}

impl<'a> ArgumentType<'a> for &'a Network
{
    type Category = AmbientArgumentType<Self>;
    fn parse_ambient(ctx: &'a impl CommandContext) -> Result<Self, CommandError>
    {
        Ok(ctx.network().as_ref())
    }
}

impl<'a> ArgumentType<'a> for &'a str
{
    type Category = PositionalArgumentType<Self>;
    fn parse_str(_ctx: &'a impl CommandContext, s: &'a str) -> Result<Self, CommandError>
    {
        Ok(s)
    }
}

impl<'a> ArgumentType<'a> for u32
{
    type Category = PositionalArgumentType<Self>;
    fn parse_str(_ctx: &'a impl CommandContext, value: &'a str) -> Result<Self, CommandError>
    {
        value.parse().map_err(|_| CommandError::UnknownError("failed to parse integer argument".to_owned()))
    }
}

impl<'a, T: ArgumentType<'a>> ArgumentType<'a> for Option<T>
{
    type Category = CustomArgumentType<Self>;
    fn parse(ctx: &'a impl CommandContext, arg: &mut ArgumentListIter<'a>) -> Result<Self, CommandError>
    {
        Ok(T::parse(ctx, arg).ok())
    }
}

impl<'a, T: ArgumentType<'a, Category=PositionalArgumentType<T>>> ArgumentType<'a> for Vec<T>
{
    type Category = CustomArgumentType<Self>;
    fn parse_custom(ctx: &'a impl CommandContext, arg: &mut ArgumentListIter<'a>) -> Result<Self, CommandError>
            where Self: ArgumentType<'a, Category = CustomArgumentType<Self>>
    {
        let mut vec = Vec::new();
        while let Some(a) = arg.next()
        {
            vec.push(T::parse_str(ctx, a)?);
        }
        Ok(vec)
    }
}