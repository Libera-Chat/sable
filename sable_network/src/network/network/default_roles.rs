use super::*;

const DEFAULT_ROLE_ID: ChannelRoleId = ChannelRoleId::new(ServerId::new(0), EpochId::new(0), 0);

impl Network {
    pub(super) fn rebuild_default_role_cache(&mut self) {
        self.cache_default_channel_roles.take();
        self.build_default_role_cache();
    }

    pub(super) fn build_default_role_cache(
        &self,
    ) -> &HashMap<state::ChannelRoleName, state::ChannelRole> {
        let mut new_cache = HashMap::new();

        for (name, flags) in &self.config.default_roles {
            let cache_role = state::ChannelRole {
                id: DEFAULT_ROLE_ID,
                name: name.clone(),
                channel: None,
                flags: *flags,
            };

            new_cache.insert(name.clone(), cache_role);
        }

        let _ = self.cache_default_channel_roles.set(new_cache);
        // We just initialised it, so unwrap won't fail
        self.cache_default_channel_roles.get().unwrap()
    }

    pub(super) fn get_default_role_cache(
        &self,
    ) -> &HashMap<state::ChannelRoleName, state::ChannelRole> {
        match self.cache_default_channel_roles.get() {
            Some(cache) => cache,
            None => self.build_default_role_cache(),
        }
    }

    pub fn find_default_role(&self, name: &state::ChannelRoleName) -> Option<wrapper::ChannelRole> {
        use super::wrapper::WrapOption;

        self.get_default_role_cache().get(name).wrap(self)
    }
}
