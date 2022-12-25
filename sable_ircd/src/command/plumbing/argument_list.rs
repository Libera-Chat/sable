use super::*;

/// A type to hold owned argument values
pub struct ArgumentList(Vec<String>);

#[derive(Clone)]
pub struct ArgumentListIter<'a>
{
    list: &'a ArgumentList,
    index: usize,
}

impl From<Vec<String>> for ArgumentList { fn from(v: Vec<String>) -> Self { Self(v) } }

impl ArgumentList
{
    pub fn iter(&self) -> ArgumentListIter
    {
        ArgumentListIter { list: self, index: 0 }
    }
/*
    pub fn get(&self, index: usize) -> Option<&str>
    {
        self.0.get(index).map(AsRef::as_ref)
    }

    pub fn len(&self) -> usize
    {
        self.0.len()
    }
*/
}

impl<I> std::ops::Index<I> for ArgumentList
    where Vec<String>: std::ops::Index<I>
{
    type Output = <Vec<String> as std::ops::Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.0[index]
    }
}

impl<'a> ArgumentListIter<'a>
{
    pub fn peek(&self) -> Option<&'a str>
    {
        self.list.0.get(self.index).map(AsRef::as_ref)
    }

    pub fn next(&mut self) -> Option<&'a str>
    {
        let idx = self.index;
        self.index += 1;
        self.list.0.get(idx).map(AsRef::as_ref)
    }
}

/// A container for the rest of the arguments, where additional parameters may or may not
/// be required depending on the processing of previous ones.
pub struct ArgList<'a>
{
    context: &'a dyn Command,
    iter: ArgumentListIter<'a>,
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
    pub fn iter(&self) -> ArgumentListIter<'a>
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
        self.iter.list.0.len() - self.iter.index
    }

    /// Return the (wrapped) command context
    pub fn context(&self) -> &'a dyn Command
    {
        self.context
    }
}

impl<'a> PositionalArgument<'a> for ArgList<'a>
{
    fn parse<'b>(context: &'a dyn Command, arg: &'b mut ArgumentListIter<'a>) -> Result<Self, CommandError>
            where 'a: 'b
    {
        Ok(Self { context, iter: arg.clone() })
    }

    fn parse_str(_ctx: &'a dyn Command, _value: &'a str) -> Result<Self, CommandError>
    {
        unreachable!()
    }
}
