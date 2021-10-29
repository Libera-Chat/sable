use super::*;
use ircd_macros::dispatch_event;
use crate::utils::FlattenResult;
use std::collections::HashSet;

fn nop<I,D>(_: I, _: &Event, _: &D) -> HandleResult { Ok(()) }

impl Server
{
    pub(super) fn pre_handle_event(&mut self, event: &Event)
    {
        let res = dispatch_event!(event => {
            NewUser => nop,
            NewUserMode => nop,
            UserModeChange => nop,
            UserQuit => self.pre_handle_quit,
            NewChannel => nop,
            NewChannelMode => nop,
            ChannelModeChange => nop,
            ChannelPermissionChange => nop,
            ChannelJoin => nop,
            ChannelPart => self.pre_handle_part,
            NewMessage => nop,
        }).flatten();
        if let Err(e) = res
        {
            error!("Error handling network event: {}", e);
        }
    }

    pub(super) fn post_handle_event(&mut self, event: &Event)
    {
        let res = dispatch_event!(event => {
            NewUser => self.handle_new_user,
            NewUserMode => nop,
            UserModeChange => self.handle_umode_change,
            UserQuit => nop,
            NewChannel => nop,
            NewChannelMode => nop,
            ChannelModeChange => self.handle_cmode_change,
            ChannelPermissionChange => self.handle_chan_perm_change,
            ChannelJoin => self.handle_join,
            ChannelPart => nop,
            NewMessage => self.handle_new_message,
        }).flatten();
        if let Err(e) = res
        {
            error!("Error handling network event: {}", e);
        }
    }

    fn handle_new_user(&mut self, user_id: UserId, _event: &Event, detail: &NewUser) -> HandleResult
    {
        if let Ok(connection) = self.connections.get_user_mut(user_id)
        {
            connection.pre_client = None;
            connection.user_id = Some(user_id);

            connection.send(&numeric::Numeric001::new_for(&self.name.to_string(), &detail.nickname, "test", &detail.nickname));
        }
        Ok(())
    }

    fn handle_umode_change(&mut self, umode_id: UModeId, _event: &Event, detail: &UserModeChange) -> HandleResult
    {
        let mode = self.net.user_mode(umode_id)?;
        let user = mode.user()?;
        let source = self.lookup_message_source(detail.changed_by)?;

        if let Ok(conn) = self.connections.get_user(user.id())
        {
            conn.send(&message::Mode::new(source.as_ref(),
                                          &user,
                                          &utils::format_umode_changes(&detail.added,&detail.removed)));
        }
        Ok(())
    }

    fn pre_handle_quit(&mut self, target: UserId, _event: &Event, detail: &UserQuit) -> HandleResult
    {
        let user = self.net.user(target)?;
        let mut to_notify = HashSet::new();

        for m1 in user.channels()
        {
            for m2 in m1.channel()?.members()
            {
                to_notify.insert(m2.user_id());
            }
        }

        for u in to_notify
        {
            if let Ok(conn) = self.connections.get_user(u)
            {
                conn.send(&message::Quit::new(&user, &detail.message));
            }
        }
        Ok(())
    }

    fn handle_cmode_change(&self, target: CModeId, _event: &Event, detail: &ChannelModeChange) -> HandleResult
    {
        let mode = self.net.channel_mode(target)?;
        let chan = mode.channel()?;
        let source = self.lookup_message_source(detail.changed_by)?;

        let changes = utils::format_cmode_changes(&detail.added, &detail.removed);
        let msg = message::Mode::new(source.as_ref(), &chan, &changes);

        for m in chan.members()
        {
            let member = m.user()?;
            if let Ok(conn) = self.connections.get_user(member.id()) {
                conn.send(&msg);
            }
        }
        Ok(())
    }

    fn handle_chan_perm_change(&self, target: MembershipId, _event: &Event, detail: &ChannelPermissionChange) -> HandleResult
    {
        let membership = self.net.membership(target)?;
        let chan = membership.channel()?;
        let target = membership.user()?;
        let source = self.lookup_message_source(detail.changed_by)?;

        let (mut changes, args) = utils::format_channel_perm_changes(&target, &detail.added, &detail.removed);

        changes += " ";
        changes += &args.join(" ");

        let msg = message::Mode::new(source.as_ref(), &chan, &changes);

        for m in chan.members()
        {
            let member = m.user()?;
            if let Ok(conn) = self.connections.get_user(member.id()) {
                conn.send(&msg);
            }
        }
        Ok(())
    }

    fn handle_join(&self, _target: MembershipId, _event: &Event, detail: &ChannelJoin) -> HandleResult
    {
        let user = self.net.user(detail.user)?;
        let channel = self.net.channel(detail.channel)?;

        for m in channel.members()
        {
            let member = m.user()?;
            if let Ok(conn) = self.connections.get_user(member.id()) {
                conn.send(&message::Join::new(&user, &channel));
            }
        }

        if let Ok(conn) = self.connections.get_user(detail.user) {
            if ! detail.permissions.is_empty()
            {
                let (mut changes, args) = utils::format_channel_perm_changes(&user, &detail.permissions, &ChannelPermissionSet::new());

                changes += " ";
                changes += &args.join(" ");

                let msg = message::Mode::new(self, &channel, &changes);
                conn.send(&msg);
            }

            irc::utils::send_channel_names(self, conn, &channel)?;
        }

        Ok(())
    }

    fn pre_handle_part(&self, target: MembershipId, _event: &Event, detail: &ChannelPart) -> HandleResult
    {
        let membership = self.net.membership(target)?;
        let source = membership.user()?;
        let channel = membership.channel()?;

        for m in channel.members()
        {
            if let Ok(conn) = self.connections.get_user(m.user()?.id())
            {
                conn.send(&message::Part::new(&source, &channel, &detail.message));
            }
        }
        Ok(())
    }

    fn handle_new_message(&self, _target: MessageId, _event: &Event, detail: &NewMessage) -> HandleResult
    {
        let source = self.net.user(detail.source)?;

        match detail.target {
            ObjectId::Channel(channel_id) => {
                let channel = self.net.channel(channel_id)?;

                for m in channel.members() {
                    let member = m.user()?;
                    if member.id() == source.id() {
                        continue;
                    }
                    if let Ok(conn) = self.connections.get_user(member.id()) {
                        conn.send(&message::Privmsg::new(&source, &channel, &detail.text));
                    }
                }
                Ok(())
            },
            ObjectId::User(user_id) => {
                let user = self.net.user(user_id)?;
                if let Ok(conn) = self.connections.get_user(user_id) {
                    conn.send(&message::Privmsg::new(&source, &user, &detail.text));

                }
                Ok(())
            },
            _ => Err(HandlerError::InternalError(format!("Message to neither user nor channel: {:?}", detail.target)))
        }
    }
}