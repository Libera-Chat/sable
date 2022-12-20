use super::*;

/// An optional argument in something other than final position. Will be interpreted as of type `T`
/// if the argument can successfully parse as that type, otherwise this argument will be empty
/// and will not consume a positional parameter
pub struct IfParses<T>(Option<T>);

impl<T> std::ops::Deref for IfParses<T>
{
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T> Into<Option<T>> for IfParses<T>
{
    fn into(self) -> Option<T> { self.0 }
}

impl<'a, T: ArgumentType<'a, Category=PositionalArgumentType<T>>> ArgumentType<'a> for IfParses<T>
{
    type Category = CustomArgumentType<Self>;
    fn parse_custom(ctx: &'a impl CommandContext, arg: &mut ArgumentListIter<'a>) -> Result<Self, CommandError>
    {
        let Some(value) = arg.peek() else { return Ok(Self(None)); };

        if let Ok(parsed) = T::parse_str(ctx, value)
        {
            // If we successfully parsed from the value, take it off the arg list
            arg.next();
            Ok(Self(Some(parsed)))
        }
        else
        {
            Ok(Self(None))
        }
    }
}

/// An optional argument in final position, which may or may not be required depending upon
/// the values of other arguments. Will be parsed as usual, but reporting of any error will
/// be deferred until the handler indicates that it is required
///
/// To access the parsed value, call `arg.require()?`
pub struct Conditional<T>(Result<T, CommandError>);

impl<T> Conditional<T>
{
    /// Return the original result of argument parsing, whether successful or not
    pub fn require(self) -> Result<T, CommandError> { self.0 }
}

impl<'a, T: ArgumentType<'a>> ArgumentType<'a> for Conditional<T>
{
    type Category = CustomArgumentType<Self>;
    fn parse_custom(ctx: &'a impl CommandContext, arg: &mut ArgumentListIter<'a>) -> Result<Self, CommandError>
    {
        Ok(Self(T::parse(ctx, arg)))
    }
}

/// A container for the rest of the arguments, where additional parameters may or may not
/// be required depending on the processing of previous ones.
pub struct ArgList<'a>
{
    context: ContextWrapper<'a>,
    values: Vec<&'a str>,
    index: usize,
}

struct ContextWrapper<'a>(&'a dyn CommandContext);

impl<'a> CommandContext for ContextWrapper<'a>
{
    fn source(&self) -> CommandSource<'_> { self.0.source() }
    fn command(&self) -> &ClientCommand { self.0.command() }
    fn server(&self) -> &Arc<ClientServer> { self.0.server() }
    fn network(&self) -> &Arc<Network> { self.0.network() }
    fn notify_error(&self, err: CommandError) { self.0.notify_error(err) }
}

impl<'a> ArgList<'a>
{
    /// Return the next argument in the list, parsing it into the required type
    /// and returning an error if it is missing or incorrect for the type
    pub fn next<'ret, T>(&mut self) -> Result<T, CommandError>
        where T: ArgumentType<'ret, Category=PositionalArgumentType<T>> + 'ret,
              Self: 'ret,
              'a: 'ret
    {
        // This is more complex than it really should be, because
        // we need &mut self to increment `index`, but the thing
        // we're returning might hold a ref to one of the things in
        // `values`, and needs to outlive the mut borrow of `self`.
        //
        // This is safe because we only ever access `values` immutably.
        let s = self.values.get(self.index).ok_or(CommandError::NotEnoughParameters)?;
        let p: *const str = *s;
        let s: &'ret str = unsafe { &*p };
        let pc: *const ContextWrapper = &self.context;
        let context: &'ret ContextWrapper = unsafe { &*pc };
        self.index += 1;
        T::parse_str(context, s)
    }

    /// Iterate over the remaining items
    pub fn iter(&self) -> impl Iterator<Item=&'a str> + '_
    {
        self.values[self.index..].iter().copied()
    }

    /// Return true if there are no arguments left un-taken
    pub fn is_empty(&self) -> bool
    {
        self.index >= self.values.len()
    }
}

impl<'a> ArgumentType<'a> for ArgList<'a>
{
    type Category = CustomArgumentType<Self>;
    fn parse_custom(ctx: &'a impl CommandContext, arg: &mut ArgumentListIter<'a>) -> Result<Self, CommandError>
            where Self: ArgumentType<'a, Category = CustomArgumentType<Self>>
    {
        let mut vec = Vec::new();
        while let Some(a) = arg.next()
        {
            vec.push(a);
        }
        Ok(Self { context: ContextWrapper(ctx), values: vec, index: 0 })
    }
}