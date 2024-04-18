use super::*;

#[command_handler("MODE")]
async fn handle_mode(
    server: &ClientServer,
    source: UserSource<'_>,
    cmd: &dyn Command,
    response: &dyn CommandResponse,
    target: TargetParameter<'_>,
    mode_str: Option<&str>,
    args: ArgList<'_>,
) -> CommandResult {
    match target {
        TargetParameter::Channel(chan) => {
            handle_channel_mode(server, &source, cmd, response, chan, mode_str, args).await
        }
        TargetParameter::User(user) => {
            handle_user_mode(server, &source, cmd, response, user, mode_str, args).await
        }
    }
}

#[derive(PartialEq)]
enum Direction {
    Add,
    Rem,
    Query,
}

impl TryFrom<char> for Direction {
    type Error = ();

    fn try_from(value: char) -> Result<Direction, Self::Error> {
        match value {
            '+' => Ok(Direction::Add),
            '-' => Ok(Direction::Rem),
            '=' => Ok(Direction::Query),
            _ => Err(()),
        }
    }
}

async fn handle_user_mode(
    server: &ClientServer,
    source: &wrapper::User<'_>,
    cmd: &dyn Command,
    response: &dyn CommandResponse,
    user: wrapper::User<'_>,
    mode_str: Option<&str>,
    _args: ArgList<'_>,
) -> CommandResult {
    if source.id() != user.id() {
        return numeric_error!(CantChangeOtherUserMode);
    }

    let mode = source.mode();

    let mut sent_unknown = false;
    let mut added = UserModeSet::new();
    let mut removed = UserModeSet::new();

    let mut dir = Direction::Query;

    let Some(mode_str) = mode_str else {
        response.numeric(make_numeric!(UserModeIs, &mode.format()));
        return Ok(());
    };

    for c in mode_str.chars() {
        if let Ok(d) = Direction::try_from(c) {
            dir = d;
        } else if let Some(flag) = UserModeSet::flag_for(c) {
            if server.policy().can_set_umode(&source, flag).is_err() {
                continue;
            }

            match dir {
                Direction::Add => {
                    added |= flag;
                }
                Direction::Rem => {
                    removed |= flag;
                }
                _ => {}
            }
        } else if !sent_unknown {
            response.numeric(make_numeric!(UnknownMode, c));
            sent_unknown = true;
        }
    }
    if !added.is_empty() || !removed.is_empty() {
        let detail = event::UserModeChange {
            changed_by: source.id().into(),
            added,
            removed,
        };
        cmd.new_event_with_response(source.id(), detail).await;
    }

    Ok(())
}

async fn handle_channel_mode(
    server: &ClientServer,
    source: &wrapper::User<'_>,
    cmd: &dyn Command,
    response: &dyn CommandResponse,
    chan: wrapper::Channel<'_>,
    mode_str: Option<&str>,
    mut args: ArgList<'_>,
) -> CommandResult {
    let mode = chan.mode();

    let Some(mode_str) = mode_str else {
        response.numeric(make_numeric!(ChannelModeIs, &chan, &mode));
        return Ok(());
    };

    let mut sent_unknown = false;
    let mut added = ChannelModeSet::new();
    let mut removed = ChannelModeSet::new();
    let mut key_change = OptionChange::<ChannelKey>::NoChange;

    let mut dir = Direction::Query;
    for c in mode_str.chars() {
        if let Ok(d) = Direction::try_from(c) {
            dir = d;
        } else if let Some(flag) = ChannelModeSet::flag_for(c) {
            server.policy().can_change_mode(source, &chan, flag)?;
            match dir {
                Direction::Add => {
                    added |= flag;
                }
                Direction::Rem => {
                    removed |= flag;
                }
                _ => {}
            }
        } else if let Some(flag) = MembershipFlagSet::flag_for(c) {
            let target = args.next::<wrapper::User>()?;
            let membership = target
                .is_in_channel(chan.id())
                .ok_or_else(|| make_numeric!(UserNotOnChannel, &target, &chan))?;
            let mut perm_added = MembershipFlagSet::new();
            let mut perm_removed = MembershipFlagSet::new();

            match dir {
                Direction::Add => {
                    server
                        .policy()
                        .can_grant_permission(source, &chan, &target, flag)?;
                    perm_added |= flag;
                }
                Direction::Rem => {
                    server
                        .policy()
                        .can_remove_permission(source, &chan, &target, flag)?;
                    perm_removed |= flag;
                }
                _ => {}
            }

            let detail = event::MembershipFlagChange {
                changed_by: source.id().into(),
                added: perm_added,
                removed: perm_removed,
            };
            cmd.new_event_with_response(membership.id(), detail).await;
        } else if let Some(list_type) = ListModeType::from_char(c) {
            let list = chan.list(list_type);

            if dir == Direction::Query || args.is_empty() {
                server.policy().can_query_list(source, &chan, list_type)?;
                send_channel_banlike_list(response, &chan, &list)?;
            } else {
                let mask = args.next::<&str>()?;

                if dir == Direction::Add {
                    server
                        .policy()
                        .can_set_ban(source, &chan, list_type, mask)?;
                    server.policy().validate_ban_mask(mask, list_type, &chan)?;

                    let detail = event::NewListModeEntry {
                        list: list.id(),
                        pattern: Pattern::new(mask.to_owned()),
                        setter: source.id(),
                    };
                    cmd.new_event_with_response(server.ids().next_list_mode_entry(), detail)
                        .await;
                } else {
                    // We've already tested for Direction::Query above, so this is definitely Remove
                    if let Some(entry) = list.entries().find(|e| e.pattern() == mask) {
                        server
                            .policy()
                            .can_unset_ban(source, &chan, list_type, mask)?;

                        let detail = event::DelListModeEntry {
                            removed_by: source.id(),
                        };
                        cmd.new_event_with_response(entry.id(), detail).await;
                    }
                }
            }
        } else if let Some(_key_type) = KeyModeType::from_char(c) {
            match dir {
                // Can't query keys
                Direction::Query => (),
                Direction::Add => {
                    let new_key = match args.next().map(ChannelKey::new_coerce)? {
                        Ok(key) => key,
                        Err(_) => return numeric_error!(InvalidKey, &chan.name()),
                    };
                    server.policy().can_set_key(source, &chan, Some(&new_key))?;
                    key_change = OptionChange::Set(new_key);
                }
                Direction::Rem => {
                    server.policy().can_set_key(source, &chan, None)?;
                    key_change = OptionChange::Unset;
                }
            }
        } else if !sent_unknown {
            response.numeric(make_numeric!(UnknownMode, c));
            sent_unknown = true;
        }
    }
    if !added.is_empty() || !removed.is_empty() || !key_change.is_no_change() {
        let detail = event::ChannelModeChange {
            changed_by: source.id().into(),
            added,
            removed,
            key_change,
        };
        cmd.new_event_with_response(chan.id(), detail).await;
    }

    Ok(())
}

fn send_channel_banlike_list(
    to: &dyn CommandResponse,
    chan: &wrapper::Channel,
    list: &wrapper::ListMode,
) -> CommandResult {
    for entry in list.entries() {
        send_banlike_list_entry(to, chan, list.list_type(), &entry)?;
    }

    send_banlike_end_numeric(to, chan, list.list_type())?;

    Ok(())
}

fn send_banlike_list_entry(
    to: &dyn CommandResponse,
    chan: &wrapper::Channel,
    list_type: ListModeType,
    entry: &wrapper::ListModeEntry,
) -> CommandResult {
    match list_type {
        ListModeType::Ban => to.numeric(make_numeric!(BanList, chan, entry)),
        ListModeType::Quiet => to.numeric(make_numeric!(QuietList, chan, entry)),
        ListModeType::Except => to.numeric(make_numeric!(ExceptList, chan, entry)),
        ListModeType::Invex => to.numeric(make_numeric!(InviteList, chan, entry)),
    }
    Ok(())
}

fn send_banlike_end_numeric(
    to: &dyn CommandResponse,
    chan: &wrapper::Channel,
    list_type: ListModeType,
) -> CommandResult {
    match list_type {
        ListModeType::Ban => to.numeric(make_numeric!(EndOfBanList, chan)),
        ListModeType::Quiet => to.numeric(make_numeric!(EndOfQuietList, chan)),
        ListModeType::Except => to.numeric(make_numeric!(EndOfExceptList, chan)),
        ListModeType::Invex => to.numeric(make_numeric!(EndOfInviteList, chan)),
    }
    Ok(())
}
