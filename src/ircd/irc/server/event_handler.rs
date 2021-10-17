use super::*;
use thiserror::Error;
use ircd_macros::dispatch_event_async;
use crate::utils::FlattenResult;
use std::collections::HashSet;

#[derive(Debug,Error)]
enum HandlerError {
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Connection error: {0}")]
    ConnectionError(#[from]ConnectionError),
    #[error("Object lookup failed: {0}")]
    LookupError(#[from] LookupError),
    #[error("Mismatched object ID type")]
    WrongIdType(#[from] WrongIdTypeError),
}

impl From<&str> for HandlerError
{
    fn from(msg: &str) -> Self { Self::InternalError(msg.to_string()) }
}

type HandleResult = Result<(), HandlerError>;

impl Server
{
    pub(super) async fn handle_event(&mut self, event: &Event)
    {
        let res = dispatch_event_async!(event => {
            NewUser => self.handle_new_user,
            UserQuit => self.handle_quit,
            NewChannel => self.handle_new_channel,
            ChannelJoin => self.handle_join,
            NewMessage => self.handle_new_message,
        }).flatten();
        if let Err(e) = res
        {
            error!("Error handling network event: {}", e);
        }
   }

    async fn handle_new_user(&mut self, user_id: UserId, _event: &Event, detail: &NewUser) -> HandleResult
    {
        if let Ok(connection) = self.connections.get_user_mut(user_id)
        {
            connection.pre_client = None;
            connection.user_id = Some(user_id);

            connection.send(&message::Numeric001::new(&self.name.to_string(), &detail.nickname, "test", &detail.nickname)).await?;
        }
        Ok(())
    }

    async fn handle_quit(&mut self, target: UserId, _event: &Event, detail: &UserQuit) -> HandleResult
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
                conn.send(&message::Quit::new(&user, &detail.message)).await?;
            }
        }
        Ok(())
    }

    /// No-op. We don't need to notify clients until somebody joins it
    async fn handle_new_channel(&self, _target: ChannelId, _event: &Event, _detail: &NewChannel) -> HandleResult
    { Ok(()) }

    async fn handle_join(&self, _target: MembershipId, _event: &Event, detail: &ChannelJoin) -> HandleResult
    {
        let user = self.net.user(detail.user)?;
        let channel = self.net.channel(detail.channel)?;

        if let Ok(conn) = self.connections.get_user(detail.user) {
            conn.send(&message::Join::new(&user, &channel)).await?;
        }

        for m in channel.members()
        {
            if m.id() == _target
            {
                continue;
            }
            let member = m.user()?;
            if let Ok(conn) = self.connections.get_user(member.id()) {
                conn.send(&message::Join::new(&user, &channel)).await?;
            }
        }
        Ok(())
    }

    async fn handle_new_message(&self, _target: MessageId, _event: &Event, detail: &NewMessage) -> HandleResult
    {
        let source = self.net.user(detail.source)?;

        match detail.target {
            ObjectId::Channel(channel_id) => {
                let channel = self.net.channel(channel_id)?;

                for m in channel.members() {
                    let member = m.user()?;
                    if let Ok(conn) = self.connections.get_user(member.id()) {
                        conn.send(&message::Privmsg::new(&source, &channel, &detail.text)).await?;
                    }
                }
                Ok(())
            },
            ObjectId::User(user_id) => {
                let user = self.net.user(user_id)?;
                if let Ok(conn) = self.connections.get_user(user_id) {
                    conn.send(&message::Privmsg::new(&source, &user, &detail.text)).await?;

                }
                Ok(())
            },
            _ => Err(HandlerError::InternalError(format!("Message to neither user nor channel: {:?}", detail.target)))
        }
    }
}