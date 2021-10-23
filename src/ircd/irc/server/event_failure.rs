use super::*;
use network::ValidationError;
use crate::utils::OrLog;
use ircd_macros::dispatch_event;

impl Server
{
    pub(super) fn handle_event_failure(&mut self, ev: &Event, er: &ValidationError)
    {
        dispatch_event!(ev => {
            NewUser => (|i,e,d| { self.failed_new_user(er, i,e,d); }),
            _ => (|_| { () })
        }).or_log("Wrong event type in failed event");
    }

    fn failed_new_user(&mut self, error: &ValidationError, user_id: UserId, _event: &Event, detail: &NewUser)
    {
        if let Ok(conn) = self.connections.get_user(user_id)
        {
            if let ValidationError::NickInUse(n) = error
            {
                if let Some(pre_client) = &conn.pre_client
                {
                    if conn.send(&numeric::NicknameInUse::new_for(&self.name, &*pre_client.borrow(), &n)).is_ok()
                    {
                        return;
                    }
                }
            }
            conn.error("Internal error in registration");
        }
        error!("Error registering user {:?} ({}!{}@{}): {}", user_id, detail.nickname, detail.username, detail.visible_hostname, error)
    }
}