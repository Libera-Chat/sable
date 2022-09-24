use crate::prelude::*;

pub trait ObjectWrapper<'a>
{
    type Underlying: 'a;
    fn wrap(network: &'a Network, obj: &'a Self::Underlying) -> Self;
    fn raw(&self) -> &'a Self::Underlying;
}

pub trait WrapOption<'a, T: ObjectWrapper<'a>>
{
    fn wrap(&self, network: &'a Network) -> Option<T>;
}

impl<'a, T: ObjectWrapper<'a>> WrapOption<'a, T> for Option<&'a T::Underlying>
{
    fn wrap(&self, network: &'a Network) -> Option<T> {
        self.map(|x| T::wrap(network, x))
    }
}

pub trait WrapResult<'a, T: ObjectWrapper<'a>, E>
{
    fn wrap(self, network: &'a Network) -> Result<T, E>;
}

impl <'a, T: ObjectWrapper<'a>, E> WrapResult<'a, T, E> for Result<&'a T::Underlying, E>
{
    fn wrap(self, network: &'a Network) -> Result<T, E> {
        Ok(T::wrap(network, self?))
    }
}

pub struct WrappedObjectIterator<'a, T: ObjectWrapper<'a>, I: Iterator<Item=&'a T::Underlying>>
{
    net: &'a Network,
    iter: I,
    _dummy: Option<&'a T>
}

impl<'a, T: ObjectWrapper<'a>, I: Iterator<Item=&'a T::Underlying>> WrappedObjectIterator<'a, T, I>
{
    pub fn new(net: &'a Network, iter: I) -> Self
    {
        Self { net, iter, _dummy: None }
    }
}

impl<'a, T: ObjectWrapper<'a>, I: Iterator<Item=&'a T::Underlying>> Iterator for WrappedObjectIterator<'a, T, I>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item>
    {
        match self.iter.next()
        {
            Some(obj) => Some(T::wrap(self.net, obj)),
            None => None
        }
    }
}

pub trait WrapIterator<'a, T: ObjectWrapper<'a>, I: Iterator<Item=&'a T::Underlying>>
{
    fn wrap(self, net: &'a Network) -> WrappedObjectIterator<'a, T, I>;
}

impl<'a, T: ObjectWrapper<'a>, I: Iterator<Item=&'a T::Underlying>> WrapIterator<'a, T, I> for I
{
    fn wrap(self, net: &'a Network) -> WrappedObjectIterator<'a, T, I> {
        WrappedObjectIterator::new(net, self)
    }
}