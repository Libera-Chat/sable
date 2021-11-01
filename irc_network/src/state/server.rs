use crate::id::*;
use crate::validated::*;

#[derive(Debug)]
pub struct Server
{
    pub id: ServerId,
    pub name: ServerName,
    pub last_ping: i64,
    pub introduced_by: EventId,
}