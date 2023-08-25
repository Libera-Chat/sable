use sable_network::prelude::state::ChannelAccessSet;

use super::*;

impl<DB: DatabaseConnection> ServicesServer<DB> {
    pub(crate) fn register_channel(
        &self,
        account_id: AccountId,
        channel_id: ChannelId,
    ) -> CommandResult {
        let net = self.node.network();

        let channel = net.channel(channel_id)?;

        let new_channel_registration = state::ChannelRegistration {
            id: self.node.ids().next_channel_registration(),
            channelname: channel.name().clone(),
        };

        let new_channel_registration =
            self.db.new_channel_registration(new_channel_registration)?;

        let new_registration_id = new_channel_registration.id;

        self.node.submit_event(
            new_registration_id,
            ChannelRegistrationUpdate {
                data: Some(new_channel_registration),
            },
        );

        let mut founder_role_id = None;

        for role in self.build_default_roles(new_registration_id) {
            let new_role = self.db.new_channel_role(role)?;

            if new_role.name == ChannelRoleName::BuiltinFounder {
                founder_role_id = Some(new_role.id);
            }

            self.node.submit_event(
                new_role.id,
                ChannelRoleUpdate {
                    data: Some(new_role),
                },
            );
        }

        let Some(founder_role_id) = founder_role_id else {
            return Err("Couldn't find built-in founder role".into());
        };

        let new_channel_access = state::ChannelAccess {
            id: ChannelAccessId::new(account_id, new_registration_id),
            role: founder_role_id,
        };

        self.db.update_channel_access(&new_channel_access)?;
        self.node.submit_event(
            new_channel_access.id,
            ChannelAccessUpdate {
                data: Some(new_channel_access),
            },
        );

        Ok(RemoteServerResponse::Success)
    }

    pub(crate) fn modify_channel_access(
        &self,
        source: AccountId,
        access_id: ChannelAccessId,
        role: Option<ChannelRoleId>,
    ) -> CommandResult {
        let net = self.node.network();
        let source_account = net.account(source)?;

        let source_access = source_account
            .has_access_in(access_id.channel())
            .ok_or(RemoteServerResponse::ChannelNotRegistered)?;

        if !source_access.has(ChannelAccessFlag::AccessEdit) {
            return Err(RemoteServerResponse::AccessDenied.into());
        }

        let target_account = net.account(access_id.account())?;

        if let Some(target_access) = target_account.has_access_in(access_id.channel()) {
            if !source_access.role()?.dominates(&target_access.role()?) {
                return Err(RemoteServerResponse::AccessDenied.into());
            }
        }

        match role {
            Some(role_id) => {
                let target_role = net.channel_role(role_id)?;

                if !source_access.role()?.dominates(&target_role) {
                    // If the source user doesn't have all the flags they're trying to grant, deny
                    return Err(RemoteServerResponse::AccessDenied.into());
                }

                let new_access = state::ChannelAccess {
                    id: access_id,
                    role: role_id,
                };

                if self.db.update_channel_access(&new_access).is_err() {
                    return Err("Database update failed".into());
                }

                self.node.submit_event(
                    access_id,
                    ChannelAccessUpdate {
                        data: Some(new_access),
                    },
                );

                Ok(RemoteServerResponse::Success)
            }
            None => {
                // We're deleting an access
                if self.db.remove_channel_access(access_id).is_err() {
                    return Err("Database update failed".into());
                }
                self.node
                    .submit_event(access_id, ChannelAccessUpdate { data: None });
                Ok(RemoteServerResponse::Success)
            }
        }
    }

    pub(crate) fn create_role(
        &self,
        source: AccountId,
        channel: ChannelRegistrationId,
        name: CustomRoleName,
        flags: ChannelAccessSet,
    ) -> CommandResult {
        let net = self.node.network();

        let source = net.account(source)?;
        let channel = net.channel_registration(channel)?;

        match source.has_access_in(channel.id()) {
            None => {
                return Err(RemoteServerResponse::AccessDenied.into());
            }
            Some(access) => {
                if !access.role()?.flags().is_set(ChannelAccessFlag::RoleEdit) {
                    return Err(RemoteServerResponse::AccessDenied.into());
                }
            }
        };

        let new_role = state::ChannelRole {
            id: self.node.ids().next_channel_role(),
            channel: Some(channel.id()),
            name: ChannelRoleName::Custom(name),
            flags,
        };

        let new_role = self.db.new_channel_role(new_role)?;

        self.node.submit_event(
            new_role.id,
            ChannelRoleUpdate {
                data: Some(new_role),
            },
        );

        Ok(RemoteServerResponse::Success)
    }

    pub(crate) fn modify_role(
        &self,
        source: AccountId,
        id: ChannelRoleId,
        flags: Option<ChannelAccessSet>,
    ) -> CommandResult {
        let net = self.node.network();

        let source = net.account(source)?;
        let existing = net.channel_role(id)?;
        let Some(channel) = existing.channel() else {
            return Err("Can't modify a builtin role".into());
        };

        match source.has_access_in(channel.id()) {
            None => {
                return Err(RemoteServerResponse::AccessDenied.into());
            }
            Some(access) => {
                if !access.role()?.flags().is_set(ChannelAccessFlag::RoleEdit) {
                    return Err(RemoteServerResponse::AccessDenied.into());
                }
            }
        };

        if let Some(flags) = flags {
            let mut role = self.db.channel_role(id)?;

            role.flags = flags;

            self.db.update_channel_role(&role)?;

            self.node
                .submit_event(role.id, ChannelRoleUpdate { data: Some(role) });
        } else {
            self.db.remove_channel_role(id)?;
            self.node.submit_event(id, ChannelRoleUpdate { data: None });
        }

        Ok(RemoteServerResponse::Success)
    }
}
