use irc_network::{
    Network,
    EventId,
    event::{
        Event,
        EventClock,
    }
};

use serde::{Serialize,Deserialize};
use tokio::sync::mpsc::Sender;

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Message
{
    NewEvent(Event),
    BulkEvents(Vec<Event>),
    SyncRequest(EventClock),
    GetEvent(Vec<EventId>),
    GetNetworkState,
    NetworkState(Network),
    Done
}

#[derive(Debug)]
pub struct Request
{
    pub(crate) response: Sender<Message>,
    pub(crate) message: Message,
}

impl Message
{
    pub fn expects_response(&self) -> bool
    {
        match self
        {
            Self::SyncRequest(_) | Self::GetEvent(_) | Self::GetNetworkState => true,
            _ => false
        }
    }
}