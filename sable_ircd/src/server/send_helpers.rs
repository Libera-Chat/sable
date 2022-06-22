use super::*;
use crate::errors::*;

use sable_network::utils::*;

impl ClientServer
{
    pub(super) fn send_to_channel_members(&self, chan: &wrapper::Channel, msg: impl MessageTypeFormat)
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

    pub(super) fn send_to_channel_members_where(&self, chan: &wrapper::Channel, msg: impl MessageTypeFormat,
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

    pub(super) fn send_to_user_if_local(&self, user: &wrapper::User, msg: impl MessageTypeFormat)
    {
        self.send_to_user_id_if_local(user.id(), msg)
    }
    pub(super) fn send_to_user_id_if_local(&self, user_id: UserId, msg: impl MessageTypeFormat)
    {
        if let Ok(conn) = self.connections.get_user(user_id) {
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
                let (mut changes, args) = format_channel_perm_changes(&user.nick(), &membership.permissions(), &MembershipFlagSet::new());

                changes += " ";
                changes += &args.join(" ");

                let msg = message::Mode::new(self, &channel, &changes);
                conn.send(&msg);
            }

            if let Some(topic) = membership.channel()?.topic()
            {
                conn.send(&numeric::TopicIs::new(&channel, topic.text())
                          .format_for(self, &user));
                conn.send(&numeric::TopicSetBy::new(&channel, topic.setter(), topic.timestamp())
                          .format_for(self, &user));
            }

            crate::utils::send_channel_names(self, conn, &user, &channel)?;
        }

        Ok(())
    }
}