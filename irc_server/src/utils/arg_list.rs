use crate::server::command_processor::*;
use super::make_numeric;

use std::slice::Iter;
use std::iter::Peekable;
use std::ops::Deref;

pub struct ArgList<'a>
{
    cmd: &'a ClientCommand<'a>,
    iter: Peekable<Iter<'a, String>>,
}

impl<'a> ArgList<'a>
{
    pub fn new(cmd: &'a ClientCommand<'a>) -> Self
    {
        Self {
            iter: cmd.args.iter().peekable(),
            cmd,
        }
    }

    pub fn next_arg(&mut self) -> Result<&String, CommandError>
    {
        Ok(self.iter.next().ok_or_else(|| make_numeric!(NotEnoughParameters, &self.cmd.command))?)
    }

    pub fn is_empty(&mut self) -> bool
    {
        self.iter.peek().is_none()
    }
/*
    pub fn iter(&mut self) -> &mut impl Iterator<Item=&'a String>
    {
        &mut self.iter
    }
*/
}

impl<'a> Deref for ArgList<'a>
{
    type Target = Peekable<Iter<'a, String>>;

    fn deref(&self) -> &Self::Target
    {
        &self.iter
    }
}