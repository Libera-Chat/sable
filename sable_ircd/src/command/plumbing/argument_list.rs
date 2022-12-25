use super::*;

#[derive(Clone)]
pub struct ArgListIter<'a>
{
    list: &'a Vec<String>,
    index: usize,
}

impl<'a> ArgListIter<'a>
{
    pub fn new(list: &'a Vec<String>) -> Self
    {
        Self { list, index: 0 }
    }

    pub fn peek(&self) -> Option<&'a str>
    {
        self.list.get(self.index).map(AsRef::as_ref)
    }

    pub fn next(&mut self) -> Option<&'a str>
    {
        let idx = self.index;
        self.index += 1;
        self.list.get(idx).map(AsRef::as_ref)
    }
}

/// A container for the rest of the arguments, where additional parameters may or may not
/// be required depending on the processing of previous ones.
pub struct ArgList<'a>
{
    context: &'a dyn Command,
    iter: ArgListIter<'a>,
}

impl<'a> ArgList<'a>
{
    /// Return the next argument in the list, parsing it into the required type
    /// and returning an error if it is missing or incorrect for the type
    pub fn next<'ret, T>(&mut self) -> Result<T, CommandError>
        where T: PositionalArgument<'ret> + 'ret,
              Self: 'ret,
              'a: 'ret
    {
        let s: &'ret str = self.iter.next().ok_or(CommandError::NotEnoughParameters)?;
        let context: &'ret dyn Command = self.context;
        T::parse_str(context, s)
    }

    /// Iterate over the remaining items
    pub fn iter(&self) -> ArgListIter<'a>
    {
        self.iter.clone()
    }

    /// Return true if there are no arguments left un-taken
    pub fn is_empty(&self) -> bool
    {
        self.iter.peek().is_none()
    }

    /// Return the number of remaining arguments
    pub fn len(&self) -> usize
    {
        self.iter.list.len() - self.iter.index
    }

    /// Return the (wrapped) command context
    pub fn context(&self) -> &'a dyn Command
    {
        self.context
    }
}

impl<'a> PositionalArgument<'a> for ArgList<'a>
{
    fn parse<'b>(context: &'a dyn Command, arg: &'b mut ArgListIter<'a>) -> Result<Self, CommandError>
            where 'a: 'b
    {
        Ok(Self { context, iter: arg.clone() })
    }

    fn parse_str(_ctx: &'a dyn Command, _value: &'a str) -> Result<Self, CommandError>
    {
        unreachable!()
    }
}
