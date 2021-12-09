use super::*;

command_handler!("MODE" => ModeHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let mut args = cmd.args.clone().into_iter();
        let mut next_arg = || { args.next().ok_or(make_numeric!(NotEnoughParameters, &cmd.command)) };

        let target = next_arg()?;

        if let Ok(cname) = ChannelName::from_str(&target)
        {
            let chan = self.server.network().channel_by_name(&cname)?;
            let mode = chan.mode()?;

            if cmd.args.len() == 1
            {
                let msg = numeric::ChannelModeIs::new(&chan, &mode);
                cmd.response(&msg)?;
            }
            else
            {
                let mut sent_unknown = false;
                let mut added = ChannelModeSet::new();
                let mut removed = ChannelModeSet::new();

                enum Direction { Add, Rem, Query }
                let mut dir = Direction::Query;
                for c in next_arg()?.chars()
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
                                let target = self.server.network().user_by_nick(&Nickname::from_str(&next_arg()?)?)?;
                                let membership = target.is_in_channel(chan.id())
                                                       .ok_or(make_numeric!(UserNotOnChannel, &target, &chan))?;
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
                            else
                            {
                                if ! sent_unknown {
                                    cmd.response(&numeric::UnknownMode::new(c))?;
                                    sent_unknown = true;
                                }
                            }
                        }
                    }
                }
                if !added.is_empty() || !removed.is_empty()
                {
                    let detail = event::ChannelModeChange { changed_by: source.id().into(), added: added, removed: removed };
                    self.action(CommandAction::state_change(mode.id(), detail))?;
                }
            }
        }
        else
        {
            if source.nick() != Nickname::from_str(&target)?
            {
                return numeric_error!(CantChangeOtherUserMode);
            }

            let mode = source.mode()?;

            let mut sent_unknown = false;
            let mut added = UserModeSet::new();
            let mut removed = UserModeSet::new();

            enum Direction { Add, Rem, Query }
            let mut dir = Direction::Query;

            for c in next_arg()?.chars()
            {
                match c
                {
                    '+' => { dir = Direction::Add; },
                    '-' => { dir = Direction::Rem; },
                    '=' => { dir = Direction::Query; },
                    _ => {
                        if let Some(flag) = UserModeSet::flag_for(c)
                        {
                            match dir {
                                Direction::Add => { added |= flag; },
                                Direction::Rem => { removed |= flag; },
                                _ => {}
                            }
                        }
                        else
                        {
                            if ! sent_unknown {
                                cmd.response(&numeric::UnknownMode::new(c))?;
                                sent_unknown = true;
                            }
                        }
                    }
                }
            }
            if !added.is_empty() || !removed.is_empty()
            {
                let detail = event::UserModeChange { changed_by: source.id().into(), added: added, removed: removed };
                self.action(CommandAction::state_change(mode.id(), detail))?;
            }
        }
        Ok(())
    }
});