use super::*;
use thiserror::Error;
use ircd_macros::dispatch_event_async;
use crate::utils::FlattenResult;

#[derive(Debug,Error)]
enum HandlerError {
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Connection error: {0}")]
    ConnectionError(#[from]ConnectionError),
    #[error("Object lookup failed: {0}")]
    LookupError(#[from] network::LookupError),
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

    async fn handle_new_user(&mut self, user_id: UserId, _event: &Event, _detail: &NewUser) -> HandleResult
    {
        if let Some(connection_id) = self.user_connections.get(&user_id)
        {
            if let Some(connection) = self.client_connections.get_mut(&connection_id)
            {
                let user = self.net.user(user_id)?;
                connection.pre_client = None;
                connection.user_id = Some(user_id);

                connection.connection.send(&format!(":{} 001 {} :Welcome to the {} IRC network, {}\r\n", 
                                                self.name,
                                                user.nick(),
                                                "test",
                                                user.nick()
                                            )).await.unwrap();
            }
        }
        Ok(())
    }

    async fn handle_quit(&mut self, target: UserId, _event: &Event, detail: &UserQuit) -> HandleResult
    {
        panic!("not implemented");
    }

    /// No-op. We don't need to notify clients until somebody joins it
    async fn handle_new_channel(&self, _target: ChannelId, _event: &Event, _detail: &NewChannel) -> HandleResult
    { Ok(()) }

    async fn handle_join(&self, _target: MembershipId, _event: &Event, detail: &ChannelJoin) -> HandleResult
    {
        let user = self.net.user(detail.user)?;
        let channel = self.net.channel(detail.channel)?;

        for m in channel.members()
        {
            if m.id() == _target
            {
                continue;
            }
            let member = m.user()?;
            if let Some(connid) = self.user_connections.get(&member.id()) {
                if let Some(conn) = self.client_connections.get(&connid) {
                    conn.connection.send(&format!(":{}!{}@{} JOIN :{}\r\n",
                                        user.nick(),
                                        user.user(),
                                        user.visible_host(),
                                        channel.name()
                    )).await?;
                }
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
                    if let Some(connid) = self.user_connections.get(&member.id()) {
                        if let Some(conn) = self.client_connections.get(&connid) {
                            conn.connection.send(&format!(":{}!{}@{} PRIVMSG {} :{}\r\n",
                                                source.nick(),
                                                source.user(),
                                                source.visible_host(),
                                                channel.name(),
                                                detail.text
                            )).await?;
                        }
                    }
                }
                Ok(())
            },
            ObjectId::User(user_id) => {
                let user = self.net.user(user_id)?;
                if let Some(connid) = self.user_connections.get(&user.id()) {
                    if let Some(conn) = self.client_connections.get(&connid) {
                        conn.connection.send(&format!(":{}!{}@{} PRIVMSG {} :{}\r\n",
                                            source.nick(),
                                            source.user(),
                                            source.visible_host(),
                                            user.nick(),
                                            detail.text
                        )).await?;
                    }
                }
                Ok(())
            },
            _ => Err(HandlerError::InternalError(format!("Message to neither user nor channel: {:?}", detail.target)))
        }
    }
}