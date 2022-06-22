use crate::modes::*;

pub fn format_umode_changes(added: &UserModeSet, removed: &UserModeSet) -> String
{
    let mut changes = String::new();
    if ! added.is_empty()
    {
        changes += "+";
        changes += &added.to_chars();
    }
    if ! removed.is_empty()
    {
        changes += "-";
        changes += &removed.to_chars();
    }

    changes
}
