use ircd_macros::object_ids;

pub type LocalId = i64;

#[derive(Debug)]
pub struct WrongIdTypeError;

impl std::fmt::Display for WrongIdTypeError
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        f.write_str("Mismatched object ID type for event")?;
        Ok(())
    }
}

impl std::error::Error for WrongIdTypeError { }

object_ids! {
    Server: (LocalId,);
    Event: (ServerId, LocalId) sequential;
    User: (ServerId, LocalId) sequential;
    Channel: (ServerId, LocalId) sequential;
    CMode: (ServerId, LocalId) sequential;
    Membership: (UserId, ChannelId);
    Listener: (LocalId,) sequential;
    Connection: (ListenerId, LocalId,) sequential;
    Message: (ServerId, LocalId) sequential;
}

impl EventId {
    pub fn server(&self) -> ServerId { self.0 }
    pub fn local(&self) -> LocalId { self.1 }
}