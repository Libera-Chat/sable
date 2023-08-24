//! Worker process for DNS and (eventually) identd checks, and library to communicate therewith.
//!
//! The [`AuthClient`] interface supports save and resume across `exec()` boundaries,
//! via the [`save_state`](AuthClient::save_state) and [`resume`](AuthClient::resume)
//! functions. The resulting `AuthClientState` can be serialised using [serde] and
//! restored in the new process image; the saving process will ensure that the required
//! file descriptors are preserved across executions.
//!
//! Obviously this only works on Unix-like systems.

mod event;
pub use event::*;

mod control;
pub use control::*;

mod auth_client;
pub use crate::auth_client::*;
