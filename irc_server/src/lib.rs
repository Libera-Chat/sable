mod utils;

pub mod policy;

pub mod errors;

pub mod server;
pub use server::Server;

pub mod client;
use client::ClientConnection;
use client::PreClient;

mod client_message;
use client_message::ClientMessage;

mod command;
pub use command::CommandResult;

mod messages;
pub use messages::message;
pub use messages::numeric;
pub use messages::MessageType;
pub use messages::Numeric;

mod build_data
{
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}