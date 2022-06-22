use super::*;

const PINGOUT_DURATION: i64 = 240;

impl Server
{
    pub(super) fn check_pings(&self)
    {
        let now = utils::now();

        let ping_detail = details::ServerPing { ts: now };
        self.submit_event(self.my_id, ping_detail);

        for server in self.net.read().servers()
        {
            if now - server.last_ping() > PINGOUT_DURATION
            {
                let quit_detail = details::ServerQuit { epoch: server.epoch() };
                self.submit_event(server.id(), quit_detail);
            }
        }
    }
}