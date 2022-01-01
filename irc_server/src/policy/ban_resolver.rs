use super::*;

pub trait BanResolver
{
    fn user_matches_entry(&self, user: &User, entry: &ListModeEntry) -> bool;

    fn user_matches_list<'a>(&self, user: &User, list: &'a ListMode<'a>) -> Option<ListModeEntry<'a>>
    {
        for entry in list.entries()
        {
            if self.user_matches_entry(user, &entry)
            {
                return Some(entry);
            }
        }
        None
    }
}

pub struct StandardBanResolver
{
}

impl StandardBanResolver
{
    pub fn new() -> Self
    {
        Self { }
    }
}

impl BanResolver for StandardBanResolver
{
    fn user_matches_entry(&self, user: &User, ban: &ListModeEntry) -> bool
    {
        let nuh = format!("{}!{}@{}", user.nick(), user.user(), user.visible_host());
        ban.pattern().matches(&nuh)
    }
}