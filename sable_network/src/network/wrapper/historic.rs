use super::*;
use crate::prelude::*;

pub enum HistoricMessageSource<'a> {
    User(&'a state::HistoricUser),
    Server(Server<'a>),
    Unknown,
}

pub enum HistoricMessageTarget<'a> {
    User(&'a state::HistoricUser),
    Channel(Channel<'a>),
    Unknown,
}
