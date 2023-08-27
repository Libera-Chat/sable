use super::*;
use client_listener::ConnectionId;

/// An action that can be triggered by a command handler.
///
/// Command handlers have only an immutable reference to the [`ClientServer`], and so
/// cannot directly change state (with limited exceptions). If handling the command
/// requires a change in state, either network or local, then this is achieved
/// by emitting a `CommandAction` which the `Server` will apply on the next
/// iteration of its event loop.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // The largest variant is also the most commonly constructed by far
pub enum CommandAction {
    /// A network state change. The target object ID and event details are provided
    /// here; the remaining [`Event`] fields are filled in by the
    /// event log.
    StateChange(ObjectId, EventDetails),

    /// Indicate that the given connection is ready to register
    RegisterClient(ConnectionId),

    /// Attach the given connection to an existing user session
    AttachToUser(ConnectionId, UserId),

    /// Update a connection's client caps
    UpdateConnectionCaps(ConnectionId, ClientCapabilitySet),

    /// Disconnect the given user. The handler should first inform the user of the reason,
    /// if appropriate.
    ///
    /// `CloseConnection` should be used for pre-clients, as they do not have a user id yet.
    DisconnectUser(UserId),

    /// Close the given connection. `DisconnectUser` should be prefered for registered clients.
    /// The handler may first inform the user of the reason.
    CloseConnection(ConnectionId),
}

impl CommandAction {
    /// Helper to create a [`CommandAction::StateChange`] variant. By passing the underlying
    /// ID and detail types, they will be converted into the corresponding enum variants.
    pub fn state_change(id: impl Into<ObjectId>, detail: impl Into<event::EventDetails>) -> Self {
        Self::StateChange(id.into(), detail.into())
    }
}
