//! Types and functions required to run a sable network server.
//!
//! [`Server`] wraps a [`ReplicatedEventLog`](sable_network::sync::ReplicatedEventLog), a
//! [`NetworkNode`](sable_network::node::NetworkNode) and a user-supplied type which implements
//! [`ServerType`] for application-specific logic. It also runs an HTTP management service to
//! handle shutdown, restart and upgrade operations, and dispatches other management commands
//! to the relevant components.
//!
//! For most servers and specialised network nodes, the [`run::run_server`] function will handle
//! all relevant setup as well as processing in-place upgrades; an application should only need
//! to parse command-line arguments and pass them to `run_server`.

pub mod config;

mod server;
pub use server::*;

mod server_type;
pub use server_type::*;

pub mod run;

mod management;

mod tracing_config;

pub use tracing_config::build_subscriber;
