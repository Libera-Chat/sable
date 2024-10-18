use super::*;

use crate::models::HistoricUser;
use rpc::NetworkHistoryUpdate;
use state::HistoricMessageSourceId;
use wrapper::HistoricMessageTarget;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

impl HistoryServer {
    pub async fn handle_history_update(&self, update: NetworkHistoryUpdate) -> anyhow::Result<()> {
        match update.change {
            NetworkStateChange::NewMessage(detail) => self.handle_new_message(detail).await,

            NetworkStateChange::NewUser(_)
            | NetworkStateChange::UserNickChange(_)
            | NetworkStateChange::UserModeChange(_)
            | NetworkStateChange::UserAwayChange(_)
            | NetworkStateChange::UserQuit(_)
            | NetworkStateChange::NewUserConnection(_)
            | NetworkStateChange::UserConnectionDisconnected(_)
            | NetworkStateChange::ChannelModeChange(_)
            | NetworkStateChange::ChannelTopicChange(_)
            | NetworkStateChange::ListModeAdded(_)
            | NetworkStateChange::ListModeRemoved(_)
            | NetworkStateChange::MembershipFlagChange(_)
            | NetworkStateChange::ChannelJoin(_)
            | NetworkStateChange::ChannelKick(_)
            | NetworkStateChange::ChannelPart(_)
            | NetworkStateChange::ChannelInvite(_)
            | NetworkStateChange::ChannelRename(_)
            | NetworkStateChange::NewServer(_)
            | NetworkStateChange::ServerQuit(_)
            | NetworkStateChange::NewAuditLogEntry(_)
            | NetworkStateChange::UserLoginChange(_)
            | NetworkStateChange::ServicesUpdate(_)
            | NetworkStateChange::EventComplete(_) => Ok(()),
        }
    }

    async fn get_or_create_historic_user(
        &self,
        huid: &HistoricUserId,
        data: &state::HistoricUser,
    ) -> anyhow::Result<crate::models::HistoricUser> {
        use crate::schema::historic_users::dsl::*;

        let mut connection_lock = self.database_connection.lock().await;

        if let Some(existing) = historic_users
            .filter(HistoricUser::with_network_id(&huid))
            .select(HistoricUser::as_select())
            .first(&mut *connection_lock)
            .await
            .optional()?
        {
            Ok(existing)
        } else {
            // There isn't a historic user in the database with that ID. That means we need to
            // (a) insert it from the data provided and (b) update the previous historic record
            // for that user id to include the timestamp it stopped being relevant
            let new_hu = crate::models::NewHistoricUser::from(data);

            // Find the most recent existing historic user for this user id
            let user_id_to_search = data.id.as_u64() as i64;
            let latest_hu_for_user: Option<crate::models::HistoricUser> = historic_users
                .filter(user_id.eq(user_id_to_search))
                .order(user_serial.desc())
                .first(&mut *connection_lock)
                .await
                .optional()?;

            if let Some(latest_hu_for_user) = latest_hu_for_user {
                let network = self.node.network();
                // Look in the network state to get the appropriate timestamp to set on the existing HU record
                if let Ok(network_hu) = network.historic_user(HistoricUserId::new(
                    UserId::new(Snowflake::from(latest_hu_for_user.user_id as u64)),
                    latest_hu_for_user.user_serial as u32,
                )) {
                    #[expect(deprecated)]
                    let timestamp_to_set = network_hu
                        .timestamp
                        .map(|ts| chrono::NaiveDateTime::from_timestamp(ts, 0));

                    // Set the timestamp
                    diesel::update(historic_users)
                        .filter(id.eq(latest_hu_for_user.id))
                        .set(last_timestamp.eq(timestamp_to_set))
                        .execute(&mut *connection_lock)
                        .await?;
                }
            }

            Ok(diesel::insert_into(historic_users)
                .values(&new_hu)
                .get_result(&mut *connection_lock)
                .await?)
        }
    }

    async fn get_or_create_channel<'a>(
        &self,
        data: wrapper::Channel<'a>,
    ) -> anyhow::Result<crate::models::Channel> {
        use crate::schema::channels::dsl::*;

        let mut connection_lock = self.database_connection.lock().await;

        let channel_id = data.id().as_u64() as i64;

        if let Some(existing) = channels
            .find(channel_id)
            .select(crate::models::Channel::as_select())
            .first(&mut *connection_lock)
            .await
            .optional()?
        {
            Ok(existing)
        } else {
            let new_channel = crate::models::Channel {
                id: channel_id,
                name: data.name().to_string(),
            };

            diesel::insert_into(channels)
                .values(&new_channel)
                .execute(&mut *connection_lock)
                .await?;
            Ok(new_channel)
        }
    }

    async fn handle_new_message(&self, new_message: update::NewMessage) -> anyhow::Result<()> {
        use crate::schema::messages::dsl::*;

        let net = self.node.network();
        let HistoricMessageSourceId::User(source_id) = new_message.source else {
            return Ok(());
        };
        let source = net.historic_user(source_id)?;

        let HistoricMessageTarget::Channel(channel) = net.message_target(&new_message.target)?
        else {
            return Ok(());
        };
        let net_message = net.message(new_message.message)?;

        let db_source = self
            .get_or_create_historic_user(&source_id, &source)
            .await?;
        let db_channel = self.get_or_create_channel(channel).await?;

        let db_message = crate::models::Message {
            id: **net_message.id(),
            source_user: db_source.id,
            target_channel: db_channel.id,
            text: net_message.text().to_string(),
        };

        let mut connection_lock = self.database_connection.lock().await;
        diesel::insert_into(messages)
            .values(&db_message)
            .execute(&mut *connection_lock)
            .await?;

        Ok(())
    }
}
