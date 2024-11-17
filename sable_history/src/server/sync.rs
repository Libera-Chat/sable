use super::*;
use crate::server::event::IntroduceHistoryServer;

impl HistoryServer {
    pub(super) async fn burst_to_network(&self) {
        // Set ourselves as the active history node
        self.node
            .submit_event(self.node.id(), IntroduceHistoryServer {});
    }
}
