use super::*;

use sable_network::network::wrapper::ObjectWrapper;

impl<DB> ServicesServer<DB>
where
    DB: DatabaseConnection,
{
    pub(super) async fn burst_to_network(&self) {
        let net = self.node.network();

        // Using unwrap here - if any of this fails, we want to fall over noisily
        // before making any network changes

        let accounts_to_sync = self.db.all_accounts().unwrap().filter(|mine| {
            if let Ok(existing) = net.account(mine.id) {
                existing.raw() != mine
            } else {
                true
            }
        });

        let accounts_to_delete = net
            .accounts()
            .filter(|existing| {
                matches!(self.db.account(existing.id()), Err(DatabaseError::NoSuchId))
            })
            .map(|obj| obj.id());

        let nicks_to_sync = self.db.all_nick_registrations().unwrap().filter(|mine| {
            if let Ok(existing) = net.nick_registration(mine.id) {
                existing.raw() != mine
            } else {
                true
            }
        });

        let nicks_to_delete = net
            .nick_registrations()
            .filter(|existing| {
                matches!(
                    self.db.nick_registration(existing.id()),
                    Err(DatabaseError::NoSuchId)
                )
            })
            .map(|obj| obj.id());

        let channels_to_sync = self.db.all_channel_registrations().unwrap().filter(|mine| {
            if let Ok(existing) = net.channel_registration(mine.id) {
                existing.raw() != mine
            } else {
                true
            }
        });

        let channels_to_delete = net
            .channel_registrations()
            .filter(|existing| {
                matches!(
                    self.db.channel_registration(existing.id()),
                    Err(DatabaseError::NoSuchId)
                )
            })
            .map(|obj| obj.id());

        let accesses_to_sync = self.db.all_channel_accesses().unwrap().filter(|mine| {
            if let Ok(existing) = net.channel_access(mine.id) {
                existing.raw() != mine
            } else {
                true
            }
        });

        let accesses_to_delete = net
            .channel_accesses()
            .filter(|existing| {
                matches!(
                    self.db.channel_access(existing.id()),
                    Err(DatabaseError::NoSuchId)
                )
            })
            .map(|obj| obj.id());

        let roles_to_sync = self.db.all_channel_roles().unwrap().filter(|mine| {
            if let Ok(existing) = net.channel_role(mine.id) {
                existing.raw() != mine
            } else {
                true
            }
        });

        let roles_to_delete = net
            .channel_roles()
            .filter(|existing| {
                matches!(
                    self.db.channel_role(existing.id()),
                    Err(DatabaseError::NoSuchId)
                )
            })
            .map(|obj| obj.id());

        for account in accounts_to_sync {
            self.node.submit_event(
                account.id,
                AccountUpdate {
                    data: Some(account),
                },
            )
        }

        for account in accounts_to_delete {
            self.node
                .submit_event(account, AccountUpdate { data: None })
        }

        for nick in nicks_to_sync {
            self.node
                .submit_event(nick.id, NickRegistrationUpdate { data: Some(nick) })
        }

        for nick in nicks_to_delete {
            self.node
                .submit_event(nick, NickRegistrationUpdate { data: None })
        }

        for channel in channels_to_sync {
            self.node.submit_event(
                channel.id,
                ChannelRegistrationUpdate {
                    data: Some(channel),
                },
            )
        }

        for channel in channels_to_delete {
            self.node
                .submit_event(channel, AccountUpdate { data: None })
        }

        for access in accesses_to_sync {
            self.node
                .submit_event(access.id, ChannelAccessUpdate { data: Some(access) })
        }

        for access in accesses_to_delete {
            self.node
                .submit_event(access, ChannelAccessUpdate { data: None })
        }

        for role in roles_to_sync {
            self.node
                .submit_event(role.id, ChannelRoleUpdate { data: Some(role) })
        }

        for role in roles_to_delete {
            self.node
                .submit_event(role, ChannelRoleUpdate { data: None })
        }

        // Finally, set ourselves as the active services node
        self.node.submit_event(
            self.node.id(),
            IntroduceServicesServer {
                sasl_mechanisms: vec!["PLAIN".to_owned()],
            },
        );
    }
}
