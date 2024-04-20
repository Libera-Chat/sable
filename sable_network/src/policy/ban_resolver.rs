use super::*;

/// A `BanResolver` contains the policy to match ban list entries
pub trait BanResolver {
    /// Determine whether the given user is matched by the given list entry.
    fn user_matches_entry(&self, user: &User, entry: &ListModeEntry) -> bool;

    /// Scan the provided list for an entry that matches the given user.
    fn user_matches_list<'a>(
        &self,
        user: &User,
        list: &'a ListMode<'a>,
    ) -> Option<ListModeEntry<'a>> {
        list.entries()
            .find(|entry| self.user_matches_entry(user, entry))
    }
}

/// Default implementation of the [`BanResolver`] trait
pub struct StandardBanResolver {}

impl StandardBanResolver {
    pub fn new() -> Self {
        Self {}
    }
}

impl BanResolver for StandardBanResolver {
    fn user_matches_entry(&self, user: &User, ban: &ListModeEntry) -> bool {
        let nuh = format!("{}!{}@{}", user.nick(), user.user(), user.visible_host());
        ban.pattern().matches(&nuh)
    }
}
