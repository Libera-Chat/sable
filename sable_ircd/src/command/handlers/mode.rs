use super::*;


#[command_handler("MODE")]
fn handle_user(server: &ClientServer, source: UserSource, cmd: &dyn Command,
               target: TargetParameter, mode_str: Option<&str>, args: ArgList) -> CommandResult
{
    match target
    {
        TargetParameter::Channel(chan) => handle_channel_mode(server, &source, cmd, chan, mode_str, args),
        TargetParameter::User(user) =>
        {
            if source.id() != user.id()
            {
                return numeric_error!(CantChangeOtherUserMode);
            }

            let mode = source.mode();

            let mut sent_unknown = false;
            let mut added = UserModeSet::new();
            let mut removed = UserModeSet::new();

            enum Direction { Add, Rem, Query }
            let mut dir = Direction::Query;

            let Some(mode_str) = mode_str else {
                cmd.numeric(make_numeric!(UserModeIs, &mode.format()));
                return Ok(())
            };

            for c in mode_str.chars()
            {
                match c
                {
                    '+' => { dir = Direction::Add; },
                    '-' => { dir = Direction::Rem; },
                    '=' => { dir = Direction::Query; },
                    _ => {
                        if let Some(flag) = UserModeSet::flag_for(c)
                        {
                            if server.policy().can_set_umode(&source, flag).is_err()
                            {
                                continue;
                            }

                            match dir {
                                Direction::Add => { added |= flag; },
                                Direction::Rem => { removed |= flag; },
                                _ => {}
                            }
                        }
                        else if ! sent_unknown
                        {
                            cmd.numeric(make_numeric!(UnknownMode, c));
                            sent_unknown = true;
                        }
                    }
                }
            }
            if !added.is_empty() || !removed.is_empty()
            {
                let detail = event::UserModeChange { changed_by: source.id().into(), added, removed };
                server.add_action(CommandAction::state_change(source.id(), detail));
            }

            Ok(())
        }
    }
}

fn handle_channel_mode(server: &ClientServer, source: &wrapper::User, cmd: &dyn Command,
        chan: wrapper::Channel, mode_str: Option<&str>, mut args: ArgList) -> CommandResult
{
    let mode = chan.mode();

    let Some(mode_str) = mode_str else {
        cmd.numeric(make_numeric!(ChannelModeIs, &chan, &mode));
        return Ok(())
    };

    let mut sent_unknown = false;
    let mut added = ChannelModeSet::new();
    let mut removed = ChannelModeSet::new();
    let mut key_change = OptionChange::<ChannelKey>::NoChange;

    #[derive(PartialEq)]
    enum Direction { Add, Rem, Query }

    let mut dir = Direction::Query;
    for c in mode_str.chars()
    {
        match c
        {
            '+' => { dir = Direction::Add; },
            '-' => { dir = Direction::Rem; },
            '=' => { dir = Direction::Query; },
            _ => {
                if let Some(flag) = ChannelModeSet::flag_for(c)
                {
                    server.policy().can_change_mode(source, &chan, flag)?;
                    match dir {
                        Direction::Add => { added |= flag; },
                        Direction::Rem => { removed |= flag; },
                        _ => {}
                    }
                }
                else if let Some(flag) = MembershipFlagSet::flag_for(c)
                {
                    let target = args.next::<wrapper::User>()?;
                    let membership = target.is_in_channel(chan.id())
                                            .ok_or_else(|| make_numeric!(UserNotOnChannel, &target, &chan))?;
                    let mut perm_added = MembershipFlagSet::new();
                    let mut perm_removed = MembershipFlagSet::new();

                    match dir {
                        Direction::Add => {
                            server.policy().can_grant_permission(source, &chan, &target, flag)?;
                            perm_added |= flag;
                        },
                        Direction::Rem => {
                            server.policy().can_remove_permission(source, &chan, &target, flag)?;
                            perm_removed |= flag;
                        },
                        _ => {}
                    }

                    let detail = event::MembershipFlagChange {
                        changed_by: source.id().into(),
                        added: perm_added,
                        removed: perm_removed,
                    };
                    server.add_action(CommandAction::state_change(membership.id(), detail));
                }
                else if let Some(list_type) = ListModeType::from_char(c)
                {
                    let list = chan.list(list_type);

                    if dir == Direction::Query || args.is_empty()
                    {
                        server.policy().can_query_list(source, &chan, list_type)?;
                        send_channel_banlike_list(cmd, &chan, &list)?;
                    }
                    else
                    {
                        let mask = args.next::<&str>()?;

                        if dir == Direction::Add
                        {
                            server.policy().can_set_ban(source, &chan, list_type, mask)?;
                            server.policy().validate_ban_mask(mask, list_type, &chan)?;

                            let detail = event::NewListModeEntry {
                                list: list.id(),
                                pattern: Pattern::new(mask.to_owned()),
                                setter: source.id()
                            };
                            server.add_action(CommandAction::state_change(server.ids().next_list_mode_entry(), detail));
                        }
                        else
                        {
                            // We've already tested for Direction::Query above, so this is definitely Remove
                            if let Some(entry) = list.entries().find(|e| e.pattern() == mask)
                            {
                                server.policy().can_unset_ban(source, &chan, list_type, mask)?;

                                let detail = event::DelListModeEntry {
                                    removed_by: source.id()
                                };
                                server.add_action(CommandAction::state_change(entry.id(), detail));
                            }
                        }
                    }
                }
                else if let Some(_key_type) = KeyModeType::from_char(c)
                {
                    match dir
                    {
                        // Can't query keys
                        Direction::Query => (),
                        Direction::Add => {
                            let new_key = args.next::<ChannelKey>()?;
                            server.policy().can_set_key(source, &chan, Some(&new_key))?;
                            key_change = OptionChange::Set(new_key);
                        }
                        Direction::Rem => {
                            server.policy().can_set_key(source, &chan, None)?;
                            key_change = OptionChange::Unset;
                        }
                    }
                }
                else if ! sent_unknown
                {
                    cmd.numeric(make_numeric!(UnknownMode, c));
                    sent_unknown = true;
                }
            }
        }
    }
    if !added.is_empty() || !removed.is_empty() || !key_change.is_no_change()
    {
        let detail = event::ChannelModeChange {
            changed_by: source.id().into(),
            added,
            removed,
            key_change,
        };
        server.add_action(CommandAction::state_change(chan.id(), detail));
    }

    Ok(())
}

fn send_channel_banlike_list(cmd: &dyn Command, chan: &wrapper::Channel, list: &wrapper::ListMode) -> CommandResult
{
    for entry in list.entries()
    {
        send_banlike_list_entry(cmd, chan, list.list_type(), &entry)?;
    }

    send_banlike_end_numeric(cmd, chan, list.list_type())?;

    Ok(())
}

fn send_banlike_list_entry(cmd: &dyn Command, chan: &wrapper::Channel, list_type: ListModeType, entry: &wrapper::ListModeEntry) -> CommandResult
{
    match list_type {
        ListModeType::Ban => cmd.numeric(make_numeric!(BanList, chan, entry)),
        ListModeType::Quiet => cmd.numeric(make_numeric!(QuietList, chan, entry)),
        ListModeType::Except => cmd.numeric(make_numeric!(ExceptList, chan, entry)),
        ListModeType::Invex => cmd.numeric(make_numeric!(InviteList, chan, entry)),
    }
    Ok(())
}

fn send_banlike_end_numeric(cmd: &dyn Command, chan: &wrapper::Channel, list_type: ListModeType) -> CommandResult
{
    match list_type {
        ListModeType::Ban => cmd.numeric(make_numeric!(EndOfBanList, chan)),
        ListModeType::Quiet => cmd.numeric(make_numeric!(EndOfQuietList, chan)),
        ListModeType::Except => cmd.numeric(make_numeric!(EndOfExceptList, chan)),
        ListModeType::Invex => cmd.numeric(make_numeric!(EndOfInviteList, chan)),
    }
    Ok(())
}
