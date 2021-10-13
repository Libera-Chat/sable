use super::*;
use thiserror::Error;

#[derive(Debug,Error)]
enum HandlerError {
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Connection error: {0}")]
    ConnectionError(#[from]ConnectionError),
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
        let res = match &event.details {
            EventDetails::NewUser(detail) => self.handle_new_user(event, detail).await,
            EventDetails::NewChannel(detail) => self.handle_new_channel(event, detail).await,
            EventDetails::ChannelJoin(detail) => self.handle_join(event, detail).await,
            EventDetails::NewMessage(detail) => self.handle_new_message(event, detail).await,
        };
        if let Err(e) = res
        {
            panic!("Error handling event: {}", e);
        }
    }

    async fn handle_new_user(&mut self, event: &Event, _detail: &NewUser) -> HandleResult
    {
        if let ObjectId::User(user_id) = event.target {
            if let Some(connection_id) = self.user_connections.get(&user_id)
            {
                if let Some(connection) = self.client_connections.get_mut(&connection_id)
                {
                    if let Some(user) = self.net.user(user_id) 
                    {
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
            }
            Ok(())
        } else {
            panic!("Got new user that isn't a user?")
        }
    }

    /// No-op. We don't need to notify clients until somebody joins it
    async fn handle_new_channel(&self, _event: &Event, _detail: &NewChannel) -> HandleResult
    { Ok(()) }

    async fn handle_join(&self, _event: &Event, detail: &ChannelJoin) -> HandleResult
    {
        let user = self.net.user(detail.user).ok_or("Join from nonexistent user")?;
        let channel = self.net.channel(detail.channel).ok_or("Join to nonexistent channel")?;

        for m in channel.members()
        {
            let member = m.user().ok_or("Nonexistent user in channel")?;
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

    async fn handle_new_message(&self, _event: &Event, detail: &NewMessage) -> HandleResult
    {
        let source = self.net.user(detail.source).ok_or("Message from nonexistent user")?;

        if let ObjectId::Channel(channel_id) = detail.target
        {
            let channel = self.net.channel(channel_id).ok_or("Message to nonexistent channel")?;

            for m in channel.members() {
                let member = m.user().ok_or("Nonexistent user in channel")?;
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
        }
        else if let ObjectId::User(user_id) = detail.target
        {
            let user = self.net.user(user_id).ok_or("Message to nonexistent user")?;
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
        }
        else {
            Err(HandlerError::InternalError(format!("Message to neither user nor channel: {:?}", detail.target)))
        }
    }
}