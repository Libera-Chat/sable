use crate::network::event::*;
use crate::network::update::*;
use crate::prelude::*;

impl Network {
    pub(super) fn load_config(
        &mut self,
        _target: ConfigId,
        _event: &Event,
        details: &details::LoadConfig,
        _updates: &dyn NetworkUpdateReceiver,
    ) {
        // If the config doesn't set debug mode, and we have a linked server with debug functionality enabled,
        // then don't load it.
        if !details.config.debug_mode
            & self
                .servers
                .values()
                .any(|s| s.flags.contains(state::ServerFlags::DEBUG))
        {
            // TODO
        } else {
            self.config = details.config.clone();
        }

        self.rebuild_default_role_cache();
        self.rebuild_alias_users();
    }
}
