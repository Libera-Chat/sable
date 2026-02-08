use wrapper::ObjectWrapper as _;

use super::*;

impl<Policy: crate::policy::PolicyService> NetworkNode<Policy> {
    pub(super) fn check_pings(&self) {
        let now = utils::now();

        let ping_detail = details::ServerPing { ts: now };
        self.submit_event(self.my_id, ping_detail);

        for server in self.net.read().servers() {
            let last_ping = server.last_ping();
            if now - last_ping > self.net.read().config().pingout_duration {
                let data = server.raw();
                tracing::info!(?last_ping, ?now, ?data, "Pinging out server");

                let quit_detail = details::ServerQuit {
                    epoch: server.epoch(),
                };
                self.submit_event(server.id(), quit_detail);
            }
        }
    }
}
