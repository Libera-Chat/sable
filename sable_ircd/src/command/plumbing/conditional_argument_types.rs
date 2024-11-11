use super::*;

/// An optional argument in something other than final position. Will be interpreted as of type `T`
/// if the argument can successfully parse as that type, otherwise this argument will be empty
/// and will not consume a positional parameter
pub struct IfParses<T>(Option<T>);

impl<T> std::ops::Deref for IfParses<T> {
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<IfParses<T>> for Option<T> {
    fn from(val: IfParses<T>) -> Self {
        val.0
    }
}

impl<'a, T: PositionalArgument<'a>> PositionalArgument<'a> for IfParses<T> {
    fn parse<'b>(ctx: &'a dyn Command, arg: &'b mut ArgListIter<'a>) -> Result<Self, CommandError>
    where
        'a: 'b,
    {
        let Some(value) = arg.peek() else {
            return Ok(Self(None));
        };

        if let Ok(parsed) = T::parse_str(ctx, value) {
            // If we successfully parsed from the value, take it off the arg list
            arg.next();
            Ok(Self(Some(parsed)))
        } else {
            Ok(Self(None))
        }
    }

    fn parse_str(_ctx: &'a dyn Command, _value: &'a str) -> Result<Self, CommandError> {
        unreachable!();
    }
}

/// An optional argument, which may or may not be required depending upon
/// the values of other arguments. Will be parsed as usual, but reporting of any error will
/// be deferred until the handler indicates that it is required
///
/// To access the parsed value, call `arg.require()?`
pub struct Conditional<T>(Result<T, CommandError>);

impl<T> Conditional<T> {
    /// Return the original result of argument parsing, whether successful or not
    pub fn require(self) -> Result<T, CommandError> {
        self.0
    }

    /// Return the argument, if it was successfully parsed
    pub fn present(self) -> Option<T> {
        self.0.ok()
    }
}

impl<'a, T: PositionalArgument<'a>> PositionalArgument<'a> for Conditional<T> {
    fn parse<'b>(ctx: &'a dyn Command, arg: &'b mut ArgListIter<'a>) -> Result<Self, CommandError>
    where
        'a: 'b,
    {
        Ok(Self(T::parse(ctx, arg)))
    }

    fn parse_str(_ctx: &'a dyn Command, _value: &'a str) -> Result<Self, CommandError> {
        unreachable!();
    }
}

impl<'a, T: AmbientArgument<'a>> AmbientArgument<'a> for Conditional<T> {
    fn load_from(ctx: &'a dyn Command) -> Result<Self, CommandError> {
        Ok(Self(T::load_from(ctx)))
    }
}
