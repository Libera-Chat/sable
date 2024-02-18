use sable_network::prelude::*;

pub struct NoOpUpdateReceiver;

impl NetworkUpdateReceiver for NoOpUpdateReceiver {
    fn notify_update(&self, _update: NetworkStateChange, _source_event: &Event) {}
}
