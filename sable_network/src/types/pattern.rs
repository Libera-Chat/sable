//! IRC-style glob matching.
//!
//! Permitted wildcards are * (match zero or more characters) and ? (match exactly one character)

use std::ops::Deref;
use std::fmt::Display;

use wildmatch::WildMatch;
use serde::{Serialize,Deserialize};

/// A wildcard pattern
#[derive(Debug,Serialize,Deserialize,Clone,PartialEq)]
pub struct Pattern(String);

impl Deref for Pattern
{
    type Target = String;

    fn deref(&self) -> &String { &self.0 }
}

impl Display for Pattern
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        self.0.fmt(f)
    }
}

impl PartialEq<String> for Pattern
{
    fn eq(&self, rhs: &String) -> bool
    {
        self.0 == *rhs
    }
}

impl PartialEq<str> for Pattern
{
    fn eq(&self, other: &str) -> bool
    {
        &self.0 == other
    }
}

impl Pattern
{
    /// Construct a `Pattern`
    pub fn new(s: String) -> Self
    {
        Self(s)
    }

    /// Test whether the given string matches this pattern. Note that this is always
    /// case-insensitive
    pub fn matches(&self, s: &str) -> bool
    {
        WildMatch::new(&self.0).matches(s)
    }
}
