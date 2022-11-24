//! Contains the message types for the synchronisation protocol

use crate::prelude::*;

use serde::{Serialize,Deserialize};
use tokio::sync::mpsc::Sender;

/// The content of a protocol message.
#[derive(Debug,Clone,Serialize,Deserialize)]
// The largest variant is NewEvent, which is also the most frequently used
#[allow(clippy::large_enum_variant)]
pub enum MessageDetail
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
    /// A message targeted to a specific server
    TargetedMessage(TargetedMessage),
    /// A response to a targeted message
    TargetedMessageResponse(rpc::RemoteServerResponse),
    /// Message was rejected because the source server has quit
    MessageRejected,
    /// Finished processing; close the connection
    Done
}

/// Details of a request/response message targeted to a particular server in the network
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct TargetedMessage
{
    /// The server from which the message originated
    pub source: String,
    /// The server to which the message is targeted
    pub target: String,
    /// Servers that have already attempted to deliver this message, and shouldn't
    /// be used as intermediaries again
    pub via: Vec<String>,
    /// The content of the message
    pub content: rpc::RemoteServerRequestType,
}

/// A single protocol message
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Message
{
    /// The server from which this message was received
    pub source_server: (ServerId, EpochId),
    /// The content of the message
    pub content: MessageDetail,
}

/// A network protocol request
#[derive(Debug)]
pub struct Request
{
    pub received_from: String,
    pub response: Sender<Message>,
    pub message: Message,
}

impl Message
{
    pub fn expects_response(&self) -> bool
    {
        matches!(self.content,
                    MessageDetail::SyncRequest(_) |
                    MessageDetail::GetEvent(_) |
                    MessageDetail::GetNetworkState
                )
    }
}