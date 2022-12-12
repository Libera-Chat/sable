use super::*;
use sable_network::prelude::*;

impl<DB: DatabaseConnection> ServicesServer<DB>
{
    pub(crate) fn register_channel(&self, account_id: AccountId, channel_id: ChannelId) -> RemoteServerResponse
    {
        let net = self.node.network();

        let Ok(channel) = net.channel(channel_id) else {
            return RemoteServerResponse::Error("Channel doesn't exist".to_string());
        };

        let new_channel_registration = state::ChannelRegistration {
            id: self.node.ids().next_channel_registration(),
            channelname: channel.name().clone()
        };

        let new_channel_access = state::ChannelAccess {
            id: ChannelAccessId::new(account_id, new_channel_registration.id),
            flags: ChannelAccessFlag::Founder | ChannelAccessFlag::Access | ChannelAccessFlag::Op
        };

        match self.db.new_channel_registration(new_channel_registration, new_channel_access)
        {
            Ok((channel_registration, channel_access)) =>
            {
                self.node.submit_event(channel_registration.id, ChannelRegistrationUpdate { data: Some(channel_registration) });
                self.node.submit_event(channel_access.id, ChannelAccessUpdate { data: Some(channel_access) });
                RemoteServerResponse::Success
            }
            Err(DatabaseError::DuplicateName) =>
            {
                RemoteServerResponse::AlreadyExists
            }
            Err(error) =>
            {
                let channel_name = channel.name();
                tracing::error!(?error, ?channel_name, "Unexpected error registering channel");
                RemoteServerResponse::Error("Unexpected error".to_string())
            }
        }
    }

    pub(crate) fn modify_channel_access(&self, source: AccountId, access_id: ChannelAccessId, flags: Option<ChannelAccessSet>) -> RemoteServerResponse
    {
        let net = self.node.network();
        let Ok(source_account) = net.account(source) else {
            return RemoteServerResponse::NoAccount;
        };

        let Some(source_access) = source_account.has_access_in(access_id.channel()) else {
            return RemoteServerResponse::ChannelNotRegistered;
        };

        let Ok(target_account) = net.account(access_id.account()) else {
            return RemoteServerResponse::Error("target user doesn't exist?".to_string());
        };

        if let Some(target_access) = target_account.has_access_in(access_id.channel())
        {
            // If the target has flags the source doesn't, deny
            let missing_flags = target_access.flags() & !source_access.flags();
            if ! missing_flags.is_empty()
            {
                return RemoteServerResponse::AccessDenied;
            }
        }

        match flags
        {
            Some(set_flags) =>
            {
                if ! source_access.flags().is_set(ChannelAccessFlag::Access)
                {
                    return RemoteServerResponse::AccessDenied;
                }

                let missing_flags = set_flags & !source_access.flags();

                if ! missing_flags.is_empty()
                {
                    // If the source user doesn't have all the flags they're trying to grant, deny
                    return RemoteServerResponse::AccessDenied;
                }

                let new_access = state::ChannelAccess {
                    id: access_id,
                    flags: set_flags,
                };
                if self.db.channel_access(access_id).is_ok()
                {
                    if self.db.update_channel_access(&new_access).is_err()
                    {
                        return RemoteServerResponse::Error("Database update failed".to_string());
                    }
                }
                else
                {
                    if self.db.new_channel_access(&new_access).is_err()
                    {
                        return RemoteServerResponse::Error("Database update failed".to_string());
                    }
                }
                self.node.submit_event(access_id, ChannelAccessUpdate { data: Some(new_access) });

                RemoteServerResponse::Success
            }
            None =>
            {
                // We're deleting an access
                if self.db.remove_channel_access(access_id).is_err()
                {
                    return RemoteServerResponse::Error("Database update failed".to_string());
                }
                self.node.submit_event(access_id, ChannelAccessUpdate { data: None });
                RemoteServerResponse::Success
            }
        }
    }
}