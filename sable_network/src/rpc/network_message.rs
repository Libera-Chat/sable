use crate::network::event::*;
use crate::network::Network;
use tokio::sync::{
    mpsc::Sender,
    oneshot,
};

/// A message emitted from the `ircd_sync` component when something
/// needs to be handled by the server logic.
#[derive(Debug)]
// The largest variant is NewEvent, which is the most commonly constructed one
#[allow(clippy::large_enum_variant)]
pub enum NetworkMessage
{
    /// An export of the current network state is required. A clone
    /// of the [Network] object should be sent across the provided channel.
    ExportNetworkState(Sender<Box<Network>>),

    /// A serialised network state has been received from the network,
    /// and should be loaded into the server's view of state.
    ImportNetworkState(Box<Network>),

    /// An event has been propagated through the network, and should be
    /// applied to the server's view of state.
    NewEvent(Event),

    /// A message to be handled by a remote node
    RemoteServerRequest(RemoteServerRequest),
}

#[derive(Debug)]
pub struct RemoteServerRequest
{
    pub req: RemoteServerRequestType,
    pub response: oneshot::Sender<RemoteServerResponse>
}

/// A message to be handled by a services node
#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub enum RemoteServerRequestType
{
    /// Simple ping for communication tests
    Ping,
}

/// Remote server response type
#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub enum RemoteServerResponse
{
    /// Operation succeeded, no output
    Success,
    /// Operation failed, with error message
    Error(String),
}

