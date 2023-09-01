use super::*;
use std::collections::HashMap;

/// A repository to store specialised message sinks so that they can be used for
/// sending state updates which should be associated with the command that
/// triggered them.
///
/// Each [`EventId`] can have at most one [`MessageSink`] implementation associated
/// with it, which must also be locked to a single [`ConnectionId`]. When processing
/// state updates caused by that event ID, when sending those updates to the specified
/// connection, then the stored `MessageSink` will be used.
pub struct MessageSinkRepository {
    sinks: HashMap<EventId, StoredLabeledSink>,
}

struct StoredLabeledSink {
    connection_id: ConnectionId,
    sink: Arc<dyn MessageSink>,
}

impl MessageSinkRepository {
    /// Construct an empty repository
    pub fn new() -> Self {
        Self {
            sinks: HashMap::new(),
        }
    }

    /// Store a message sink to be used later
    pub fn store(
        &mut self,
        event_id: EventId,
        connection_id: ConnectionId,
        sink: Arc<dyn MessageSink>,
    ) {
        self.sinks.insert(
            event_id,
            StoredLabeledSink {
                connection_id,
                sink,
            },
        );
    }

    /// Retrieve a message sink, if one is stored, for notifying the given connection about the given event
    pub fn get(&self, event_id: &EventId, connection_id: ConnectionId) -> Option<&dyn MessageSink> {
        if let Some(stored) = self.sinks.get(event_id) {
            if stored.connection_id == connection_id {
                Some(stored.sink.as_ref())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Remove the stored sink for the given event
    pub fn remove(&mut self, event_id: &EventId) {
        self.sinks.remove(event_id);
    }
}
