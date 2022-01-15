use irc_network::event::*;
use irc_network::Network;
use tokio::{
    sync::mpsc::Sender,
};

/// A message emitted from the `ircd_sync` component when something
/// needs to be handled by the server logic.
#[derive(Debug)]
pub enum NetworkMessage
{
    /// An export of the current network state is required. A clone
    /// of the [Network] object should be sent across the provided channel.
    ExportNetworkState(Sender<Network>),

    /// A serialised network state has been received from the network,
    /// and should be loaded into the server's view of state.
    ImportNetworkState(Network),

    /// An event has been propagated through the network, and should be
    /// applied to the server's view of state.
    NewEvent(Event),
}
