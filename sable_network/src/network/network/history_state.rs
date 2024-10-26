use super::Network;
use crate::network::event::*;
use crate::network::update::*;
use crate::prelude::*;

impl Network {
    pub(super) fn introduce_history_server(
        &mut self,
        target: ServerId,
        _event: &Event,
        _update: &IntroduceHistoryServer,
        _updates: &dyn NetworkUpdateReceiver,
    ) {
        self.current_history_server_id = Some(target);
    }
}
