use irc_network::wrapper::*;
use irc_network::flags::*;

pub fn format_cmode_changes(added: &ChannelModeSet, removed: &ChannelModeSet) -> String
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

pub fn format_channel_perm_changes(target: &User, added: &ChannelPermissionSet, removed: &ChannelPermissionSet) -> (String, Vec<String>)
{
    let mut changes = String::new();
    let mut args = Vec::new();

    if ! added.is_empty()
    {
        changes += "+";
        for (flag,modechar,_) in ChannelPermissionSet::all()
        {
            if added.is_set(flag) {
                changes += &modechar.to_string();
                args.push(target.nick().to_string());
            }
        }
    }
    if ! removed.is_empty()
    {
        changes += "-";
        for (flag,modechar,_) in ChannelPermissionSet::all()
        {
            if removed.is_set(flag) {
                changes += &modechar.to_string();
                args.push(target.nick().to_string());
            }
        }
    }

    (changes, args)
}