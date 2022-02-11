use std::cmp::{Ord,PartialOrd};
use serde::{Serialize,Deserialize};

/// Describes an optional change to an optional value.
#[derive(Debug,Clone,Copy,Hash,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
pub enum OptionChange<T>
{
    NoChange,
    Unset,
    Set(T)
}

impl<T> OptionChange<T>
{
    pub fn is_set(&self) -> bool
    {
        matches!(self, Self::Set(_))
    }

    pub fn is_unset(&self) -> bool
    {
        matches!(self, Self::Unset)
    }

    pub fn is_no_change(&self) -> bool
    {
        matches!(self, Self::NoChange)
    }
}