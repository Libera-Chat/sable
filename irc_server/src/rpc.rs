use irc_network::event::*;
use irc_network::Network;
use tokio::{
    sync::mpsc::Sender,
};

#[derive(Debug)]
pub enum ServerRpcMessage
{
    ExportNetworkState(Sender<Network>),
    ImportNetworkState(Network),
    NewEvent(Event),
}