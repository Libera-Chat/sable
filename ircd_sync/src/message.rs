//! Contains the message types for the synchronisation protocol

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

/// A single protocol message.
#[derive(Debug,Clone,Serialize,Deserialize)]
// The largest variant is NewEvent, which is also the most frequently used
#[allow(clippy::large_enum_variant)]
pub enum Message
{
    /// A new event has been created
    NewEvent(Event),
    /// Used in response to `SyncRequest` or `GetEvent` to transmit multiple
    /// [Event]s at once.
    BulkEvents(Vec<Event>),
    /// Request for all events not contained in the provided event clock
    SyncRequest(EventClock),
    /// Request for specific event IDs
    GetEvent(Vec<EventId>),
    /// Request to export the current network state
    GetNetworkState,
    /// Response containing the current network state
    NetworkState(Box<Network>),
    /// Close the connection
    Done
}

/// A network protocol request
#[derive(Debug)]
pub struct Request
{
    pub response: Sender<Message>,
    pub message: Message,
}

impl Message
{
    pub fn expects_response(&self) -> bool
    {
        matches!(self, Self::SyncRequest(_) | Self::GetEvent(_) | Self::GetNetworkState)
    }
}