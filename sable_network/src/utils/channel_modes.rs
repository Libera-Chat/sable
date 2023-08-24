use crate::prelude::*;
use update::*;

fn has_plus(changes: &ChannelModeChange) -> bool {
    (!changes.added.is_empty()) || changes.key_change.is_set()
}

fn has_minus(changes: &ChannelModeChange) -> bool {
    (!changes.removed.is_empty()) || changes.key_change.is_unset()
}

pub fn format_cmode_changes(detail: &ChannelModeChange) -> (String, Vec<String>) {
    let mut changes = String::new();
    let mut params = Vec::new();
    if has_plus(detail) {
        changes += "+";
        changes += &detail.added.to_chars();
        if let OptionChange::Set(new_key) = detail.key_change {
            changes.push(KeyModeType::Key.mode_letter());
            params.push(new_key.to_string());
        }
    }
    if has_minus(detail) {
        changes += "-";
        changes += &detail.removed.to_chars();
        if detail.key_change.is_unset() {
            changes.push(KeyModeType::Key.mode_letter());
            params.push("*".to_string());
        }
    }

    (changes, params)
}

pub fn format_channel_perm_changes(
    nick: &Nickname,
    added: &MembershipFlagSet,
    removed: &MembershipFlagSet,
) -> (String, Vec<String>) {
    let mut changes = String::new();
    let mut args = Vec::new();

    if !added.is_empty() {
        changes += "+";
        for (flag, modechar, _) in MembershipFlagSet::all() {
            if added.is_set(flag) {
                changes += &modechar.to_string();
                args.push(nick.to_string());
            }
        }
    }
    if !removed.is_empty() {
        changes += "-";
        for (flag, modechar, _) in MembershipFlagSet::all() {
            if removed.is_set(flag) {
                changes += &modechar.to_string();
                args.push(nick.to_string());
            }
        }
    }

    (changes, args)
}
