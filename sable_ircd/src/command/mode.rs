use super::*;
use super::ArgList;
use irc_strings::matches::Pattern;

command_handler!("MODE" => ModeHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let mut args = ArgList::new(cmd);

        let target = args.next_arg()?;

        if let Ok(cname) = ChannelName::from_str(target)
        {
            self.handle_channel_mode(source, cmd, cname, &mut args)?;
        }
        else
        {
            if source.nick() != Nickname::from_str(target)?
            {
                return numeric_error!(CantChangeOtherUserMode);
            }

            let mode = source.mode();

            let mut sent_unknown = false;
            let mut added = UserModeSet::new();
            let mut removed = UserModeSet::new();

            enum Direction { Add, Rem, Query }
            let mut dir = Direction::Query;

            if args.is_empty()
            {
                cmd.response(&numeric::UserModeIs::new(&mode.format()))?;
                return Ok(())
            }

            for c in args.next_arg()?.chars()
            {
                match c
                {
                    '+' => { dir = Direction::Add; },
                    '-' => { dir = Direction::Rem; },
                    '=' => { dir = Direction::Query; },
                    _ => {
                        if let Some(flag) = UserModeSet::flag_for(c)
                        {
                            if self.server.policy().can_set_umode(source, flag).is_err()
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
                            cmd.response(&numeric::UnknownMode::new(c))?;
                            sent_unknown = true;
                        }
                    }
                }
            }
            if !added.is_empty() || !removed.is_empty()
            {
                let detail = event::UserModeChange { changed_by: source.id().into(), added, removed };
                self.action(CommandAction::state_change(source.id(), detail))?;
            }
        }
        Ok(())
    }
});

impl ModeHandler<'_>
{
    fn handle_channel_mode(&mut self, source: &wrapper::User, cmd: &ClientCommand, cname: ChannelName, args: &mut ArgList) -> CommandResult
    {
        let net = self.server.network();
        let chan = net.channel_by_name(&cname)?;
        let mode = chan.mode();

        if args.is_empty()
        {
            let msg = numeric::ChannelModeIs::new(&chan, &mode);
            cmd.response(&msg)?;
        }
        else
        {
            let mut sent_unknown = false;
            let mut added = ChannelModeSet::new();
            let mut removed = ChannelModeSet::new();
            let mut key_change = OptionChange::<ChannelKey>::NoChange;

            #[derive(PartialEq)]
            enum Direction { Add, Rem, Query }

            let mut dir = Direction::Query;
            for c in args.next_arg()?.clone().chars()
            {
                match c
                {
                    '+' => { dir = Direction::Add; },
                    '-' => { dir = Direction::Rem; },
                    '=' => { dir = Direction::Query; },
                    _ => {
                        if let Some(flag) = ChannelModeSet::flag_for(c)
                        {
                            self.server.policy().can_change_mode(source, &chan, flag)?;
                            match dir {
                                Direction::Add => { added |= flag; },
                                Direction::Rem => { removed |= flag; },
                                _ => {}
                            }
                        }
                        else if let Some(flag) = MembershipFlagSet::flag_for(c)
                        {
                            let target = net.user_by_nick(&Nickname::from_str(args.next_arg()?)?)?;
                            let membership = target.is_in_channel(chan.id())
                                                   .ok_or_else(|| make_numeric!(UserNotOnChannel, &target, &chan))?;
                            let mut perm_added = MembershipFlagSet::new();
                            let mut perm_removed = MembershipFlagSet::new();

                            match dir {
                                Direction::Add => {
                                    self.server.policy().can_grant_permission(source, &chan, &target, flag)?;
                                    perm_added |= flag;
                                },
                                Direction::Rem => {
                                    self.server.policy().can_remove_permission(source, &chan, &target, flag)?;
                                    perm_removed |= flag;
                                },
                                _ => {}
                            }

                            let detail = event::MembershipFlagChange {
                                changed_by: source.id().into(),
                                added: perm_added,
                                removed: perm_removed,
                            };
                            self.action(CommandAction::state_change(membership.id(), detail))?;
                        }
                        else if let Some(list_type) = ListModeType::from_char(c)
                        {
                            let list = chan.list(list_type);

                            if dir == Direction::Query || args.is_empty()
                            {
                                self.server.policy().can_query_list(source, &chan, list_type)?;
                                self.send_channel_banlike_list(cmd, &chan, &list)?;
                            }
                            else
                            {
                                let mask = args.next_arg()?;

                                if dir == Direction::Add
                                {
                                    self.server.policy().can_set_ban(source, &chan, list_type, mask)?;
                                    self.server.policy().validate_ban_mask(mask, list_type, &chan)?;

                                    let detail = event::NewListModeEntry {
                                        list: list.id(),
                                        pattern: Pattern::new(mask.clone()),
                                        setter: source.id()
                                    };
                                    self.action(CommandAction::state_change(self.server.ids().next_list_mode_entry(), detail))?;
                                }
                                else
                                {
                                    // We've already tested for Direction::Query above, so this is definitely Remove
                                    if let Some(entry) = list.entries().find(|e| e.pattern() == mask)
                                    {
                                        self.server.policy().can_unset_ban(source, &chan, list_type, mask)?;

                                        let detail = event::DelListModeEntry {
                                            removed_by: source.id()
                                        };
                                        self.action(CommandAction::state_change(entry.id(), detail))?;
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
                                    let new_key = ChannelKey::new_coerce(args.next_arg()?);
                                    self.server.policy().can_set_key(source, &chan, Some(&new_key))?;
                                    key_change = OptionChange::Set(new_key);
                                }
                                Direction::Rem => {
                                    self.server.policy().can_set_key(source, &chan, None)?;
                                    key_change = OptionChange::Unset;
                                }
                            }
                        }
                        else if ! sent_unknown
                        {
                            cmd.response(&numeric::UnknownMode::new(c))?;
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
                self.action(CommandAction::state_change(chan.id(), detail))?;
            }
        }
        Ok(())
    }

    fn send_channel_banlike_list(&self, cmd: &ClientCommand, chan: &wrapper::Channel, list: &wrapper::ListMode) -> CommandResult
    {
        for entry in list.entries()
        {
            self.send_banlike_list_entry(cmd, chan, list.list_type(), &entry)?;
        }

        self.send_banlike_end_numeric(cmd, chan, list.list_type())?;

        Ok(())
    }

    fn send_banlike_list_entry(&self, cmd: &ClientCommand, chan: &wrapper::Channel, list_type: ListModeType, entry: &wrapper::ListModeEntry) -> CommandResult
    {
        match list_type {
            ListModeType::Ban => cmd.response(&numeric::BanList::new(chan, entry)),
            ListModeType::Quiet => cmd.response(&numeric::QuietList::new(chan, entry)),
            ListModeType::Except => cmd.response(&numeric::ExceptList::new(chan, entry)),
            ListModeType::Invex => cmd.response(&numeric::InviteList::new(chan, entry)),
        }
    }

    fn send_banlike_end_numeric(&self, cmd: &ClientCommand, chan: &wrapper::Channel, list_type: ListModeType) -> CommandResult
    {
        match list_type {
            ListModeType::Ban => cmd.response(&numeric::EndOfBanList::new(chan)),
            ListModeType::Quiet => cmd.response(&numeric::EndOfQuietList::new(chan)),
            ListModeType::Except => cmd.response(&numeric::EndOfExceptList::new(chan)),
            ListModeType::Invex => cmd.response(&numeric::EndOfInviteList::new(chan)),
        }
    }
}