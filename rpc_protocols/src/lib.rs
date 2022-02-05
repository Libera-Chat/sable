//! Defines types used to communicate between various components of the
//! server architecture.
//!
//! This is a separate crate in order to avoid `irc_network` depending
//! on tokio.

mod network_message;
pub use network_message::*;

mod log_update;
pub use log_update::*;

mod shutdown_action;
pub use shutdown_action::*;
