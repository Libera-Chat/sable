use super::*;

pub struct ArgumentList(Vec<String>);

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

    pub fn get(&self, index: usize) -> Option<&str>
    {
        self.0.get(index).map(AsRef::as_ref)
    }

    pub fn len(&self) -> usize
    {
        self.0.len()
    }
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