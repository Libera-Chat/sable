use super::*;

impl Server
{
    pub(super) fn send_to_channel_members(&self, chan: &wrapper::Channel, msg: impl MessageType)
    {
        for m in chan.members()
        {
            if let Ok(member) = m.user()
            {
                if let Ok(conn) = self.connections.get_user(member.id()) {
                    conn.send(&msg);
                }
            }
        }
    }

    pub(super) fn send_to_channel_members_where(&self, chan: &wrapper::Channel, msg: impl MessageType,
                        pred: impl Fn(&wrapper::Membership) -> bool
                    )
    {
        for m in chan.members()
        {
            if pred(&m)
            {
                if let Ok(member) = m.user()
                {
                    if let Ok(conn) = self.connections.get_user(member.id()) {
                        conn.send(&msg);
                    }
                }
            }
        }
    }
}