use super::*;

use std::sync::mpsc::{channel, Receiver, Sender};

pub struct SavedUpdateReceiver {
    sender: Sender<SavedNetworkStateChange>,
    receiver: Receiver<SavedNetworkStateChange>,
}

struct SavedNetworkStateChange {
    update: NetworkStateChange,
    source_event: event::Event,
}

impl NetworkUpdateReceiver for SavedUpdateReceiver {
    fn notify_update(&self, update: NetworkStateChange, source_event: &event::Event) {
        self.sender
            .send(SavedNetworkStateChange {
                update,
                source_event: source_event.clone(),
            })
            .expect("failed to save network state change");
    }
}

impl SavedUpdateReceiver {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self { sender, receiver }
    }

    pub fn playback(&mut self, into: &impl NetworkUpdateReceiver) {
        while let Ok(saved) = self.receiver.try_recv() {
            into.notify_update(saved.update, &saved.source_event);
        }
    }
}
