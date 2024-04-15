use super::*;

impl Network {
    pub(super) fn build_alias_users(&self) -> &HashMap<Nickname, state::User> {
        let mut alias_users = HashMap::new();

        for (id, user_config) in self.config.alias_users.iter().enumerate() {
            // Create alias users with invalid ID and server ID
            alias_users.insert(
                user_config.nick.clone(),
                state::User {
                    id: UserId::new(ServerId::new(0), EpochId::new(0), id as LocalId),
                    user: user_config.user.clone(),
                    visible_host: user_config.host.clone(),
                    realname: user_config.realname.clone(),
                    mode: state::UserMode::new(UserModeSet::new()),
                    oper_privileges: None,
                    away_reason: None, // Never away
                    account: None,
                    session_key: None,
                },
            );
        }

        self.alias_users.set(alias_users).ok();
        self.alias_users.get().unwrap()
    }

    pub(super) fn rebuild_alias_users(&mut self) {
        self.alias_users.take();
        self.build_alias_users();
    }

    pub(super) fn get_alias_users(&self) -> &HashMap<Nickname, state::User> {
        match self.alias_users.get() {
            Some(cache) => cache,
            None => self.build_alias_users(),
        }
    }

    pub(super) fn find_alias_user_with_id(&self, id: UserId) -> Option<(&Nickname, &state::User)> {
        self.get_alias_users()
            .iter()
            .find(|(_, user)| user.id == id)
    }

    pub fn user_is_alias(&self, id: UserId) -> Option<&config::AliasUser> {
        if id.server() == ServerId::new(0) && id.epoch() == EpochId::new(0) {
            self.config.alias_users.get(id.local() as usize)
        } else {
            None
        }
    }
}
