use super::*;

command_handler!("MODE" => ModeHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let target = cmd.args[0].clone();
        if let Ok(cname) = ChannelName::new(target)
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
                for c in cmd.args[1].chars()
                {
                    match c
                    {
                        '+' => { dir = Direction::Add; },
                        '-' => { dir = Direction::Rem; },
                        '=' => { dir = Direction::Query; },
                        _ => {
                            match ChannelModeSet::flag_for(c)
                            {
                                Some(flag) => {
                                    match dir {
                                        Direction::Add => { added |= flag; },
                                        Direction::Rem => { removed |= flag; },
                                        _ => {}
                                    }
                                },
                                None => {
                                    if ! sent_unknown {
                                        cmd.response(&numeric::UnknownMode::new(c))?;
                                        sent_unknown = true;
                                    }
                                }
                            }
                        }
                    }
                }
                if !added.is_empty() || !removed.is_empty()
                {
                    let detail = event::ChannelModeChange { changed_by: source.id().into(), added: added, removed: removed };
                    let event = self.server.create_event(mode.id(), detail);
                    self.action(CommandAction::StateChange(event))?;
                }
            }
        }
        else
        {
            log::error!("User modes not implemented");
        }
        Ok(())
    }
});