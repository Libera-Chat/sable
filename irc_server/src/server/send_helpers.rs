use super::*;
use crate::errors::*;

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

    pub(super) fn send_to_user_if_local(&self, user: &wrapper::User, msg: impl MessageType)
    {
        if let Ok(conn) = self.connections.get_user(user.id()) {
            conn.send(&msg);
        }
    }

    pub(super) fn notify_joining_user(&self, membership: &wrapper::Membership) -> HandleResult
    {
        let user = membership.user()?;
        let channel = membership.channel()?;

        if let Ok(conn) = self.connections.get_user(user.id()) {
            if ! membership.permissions().is_empty()
            {
                let (mut changes, args) = utils::format_channel_perm_changes(&user, &membership.permissions(), &MembershipFlagSet::new());

                changes += " ";
                changes += &args.join(" ");

                let msg = message::Mode::new(self, &channel, &changes);
                conn.send(&msg);
            }

            if let Ok(topic) = self.net.topic_for_channel(channel.id())
            {
                conn.send(&numeric::TopicIs::new(&channel, topic.text())
                          .format_for(self, &user));
                conn.send(&numeric::TopicSetBy::new(&channel, topic.setter(), topic.timestamp())
                          .format_for(self, &user));
            }

            crate::utils::send_channel_names(self, conn, &channel)?;
        }

        Ok(())
    }
}