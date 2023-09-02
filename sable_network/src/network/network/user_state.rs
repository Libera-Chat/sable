use super::*;
use crate::{network::state_utils, prelude::state::UserSessionKey};

impl Network {
    pub(super) fn remove_user(&mut self, id: UserId, message: String) -> Option<update::UserQuit> {
        if let Some(user) = self.users.remove(&id) {
            // Because HashMap::drain_filter isn't stable yet...
            let mut removed_memberships = Vec::new();
            let mut retained_memberships = HashMap::new();

            for (id, membership) in self.memberships.drain() {
                if membership.user == user.id {
                    removed_memberships.push(membership);
                } else {
                    retained_memberships.insert(id, membership);
                }
            }
            self.memberships = retained_memberships;

            let removed_nickname = if let Ok(binding) = self.nick_binding_for_user(user.id) {
                let nick = binding.nick();
                self.nick_bindings.remove(&nick);
                nick
            } else {
                state_utils::hashed_nick_for(user.id)
            };

            Some(update::UserQuit {
                // We can't use `translate_historic_user` because we've already removed the nick binding
                user: HistoricUser {
                    user,
                    nickname: removed_nickname,
                },
                nickname: removed_nickname,
                message,
                memberships: removed_memberships,
            })
        } else {
            None
        }
    }

    fn collide_user(
        &mut self,
        user_id: UserId,
        from: Nickname,
        trigger: &Event,
        updates: &dyn NetworkUpdateReceiver,
    ) {
        if let Some(user) = self.users.get(&user_id) {
            let new_nick = state_utils::hashed_nick_for(user_id);
            if let Some(existing_id_binding) = self.nick_bindings.remove(&new_nick) {
                // The hash-based nick is already in use.
                // If the clock of the event we're currently processing depends on the event
                // that created the existing binding, then it's safe to only kill this user
                // and leave the existing one. If it doesn't, then we don't know that all servers
                // will process these two in the same order, and we need to kill both.

                if trigger.clock.contains(existing_id_binding.created) {
                    // The event we're processing depends on the one that created the existing binding.
                    // Kill only the current user (below), and put the existing binding back.
                    self.nick_bindings
                        .insert(existing_id_binding.nick, existing_id_binding);
                } else {
                    // The event we're processing does not depend on the one that created the
                    // existing binding. Kill both users, and drop the old binding.
                    if let Some(update) =
                        self.remove_user(existing_id_binding.user, "Nickname collision".to_string())
                    {
                        updates.notify(update, trigger);
                    }
                }

                // Whichever way the above test went, we need to kill the newer user.
                if let Some(update) = self.remove_user(user_id, "Nickname collision".to_string()) {
                    updates.notify(update, trigger);
                }
            } else {
                // The ID-based nick isn't bound. Do so.
                let new_binding =
                    state::NickBinding::new(new_nick, user_id, trigger.timestamp, trigger.id);
                let update = UserNickChange {
                    user: user.clone(),
                    old_nick: from,
                    new_nick: new_binding.nick,
                };
                self.nick_bindings.insert(new_nick, new_binding);
                updates.notify(update, trigger);
            }
        }
    }

    fn do_bind_nickname(
        &mut self,
        target: NicknameId,
        user: UserId,
        old_nick: Nickname,
        event: &Event,
        updates: &dyn NetworkUpdateReceiver,
    ) {
        // If an alias users exists with this nickname, collide the user attempting to bind it
        if self.get_alias_users().contains_key(target.nick()) {
            self.collide_user(user, old_nick, event, updates);
            return;
        }
        if let Some(existing) = self.nick_bindings.remove(target.nick()) {
            // Conflict. This can only happen if neither of the event that created the existing binding,
            // and the event we're processing now, depends on the other (if they did either way, then the
            // server emitting the later event would know about the earlier and refuse to bind). Since we
            // can't use the  dependency-order to resolve, the timestamp is the best fallback we have,
            // followed by lexicographical comparison of user IDs as a tie-breaker.
            if existing.timestamp < event.timestamp
                || (existing.timestamp == event.timestamp && existing.user < user)
            {
                // The existing one wins. Collide the user attempting to bind,
                // and put the existing binding back.
                self.collide_user(user, old_nick, event, updates);
                self.nick_bindings.insert(existing.nick, existing);
                return;
            } else {
                // The new one wins. Collide the existing user.
                self.collide_user(existing.user, existing.nick, event, updates);
            }
        }

        // If we get here, then either there was no conflict or the existing binding has been removed,
        // and we can continue
        let new_binding = state::NickBinding::new(*target.nick(), user, event.timestamp, event.id);
        if let Some(user_object) = self.users.get(&user) {
            let new_nick = new_binding.nick;
            self.nick_bindings.insert(new_nick, new_binding);

            // Emit UserNickChange update if a nick change happens as a result of this rebinding.
            if old_nick.value() != new_nick.value() {
                let update = UserNickChange {
                    user: user_object.clone(),
                    old_nick,
                    new_nick,
                };
                updates.notify(update, event);
            }
        }
    }

    pub(super) fn bind_nickname(
        &mut self,
        target: NicknameId,
        event: &Event,
        binding: &details::BindNickname,
        updates: &dyn NetworkUpdateReceiver,
    ) {
        // Check for an existing binding we need to remove. If there isn't, fall back to the ID-based hash nick
        // for the nick change notification
        if let Ok(user) = self.user(binding.user) {
            let prev_nick = user.nick();
            self.nick_bindings.remove(&prev_nick);
            self.do_bind_nickname(target, binding.user, prev_nick, event, updates);
        } else {
            tracing::error!(
                "Tried to bind nickname {:?} to nonexistent user {:?}",
                target,
                binding.user
            );
        }
    }

    pub(super) fn new_user(
        &mut self,
        target: UserId,
        event: &Event,
        detail: &details::NewUser,
        updates: &dyn NetworkUpdateReceiver,
    ) {
        let user = state::User::new(
            target,
            detail.server,
            detail.username,
            detail.visible_hostname,
            detail.realname.clone(),
            detail.mode.clone(),
            detail.account.clone(),
        );

        // First insert the user (with no nickname yet) so that the nick binding can see
        // a user to bind to
        self.users.insert(target, user.clone());

        // Then insert the nick binding to associate a nickname
        // If there's a nick collision, we need to use the nick provided by the user as the 'from' nickname
        // to send to that user when notifying of the change, so provide that here
        self.do_bind_nickname(
            NicknameId::new(detail.nickname),
            target,
            detail.nickname,
            event,
            updates,
        );

        let update = update::NewUser {
            user: self.translate_historic_user(user),
        };
        updates.notify(update, event);
    }

    pub(super) fn validate_new_user(
        &self,
        _target: UserId,
        user: &details::NewUser,
    ) -> ValidationResult {
        if self.nick_bindings.contains_key(&user.nickname) {
            Err(ValidationError::NickInUse(user.nickname))
        } else {
            Ok(())
        }
    }

    pub(super) fn user_away(
        &mut self,
        target: UserId,
        event: &Event,
        update: &UserAway,
        updates: &dyn NetworkUpdateReceiver,
    ) {
        if let Some(user) = self.users.get_mut(&target) {
            let new_reason = update.reason.clone();
            let mut old_reason = new_reason.clone();
            std::mem::swap(&mut user.away_reason, &mut old_reason);

            let update_user = user.clone();

            let update = update::UserAwayChange {
                user: self.translate_historic_user(update_user),
                old_reason,
                new_reason,
            };

            updates.notify(update, event);
        }
    }

    pub(super) fn user_mode_change(
        &mut self,
        target: UserId,
        event: &Event,
        mode: &details::UserModeChange,
        updates: &dyn NetworkUpdateReceiver,
    ) {
        if let Some(user) = self.users.get_mut(&target) {
            user.mode.modes |= mode.added;
            user.mode.modes &= !mode.removed;

            let update_user = user.clone();

            updates.notify(
                update::UserModeChange {
                    user: self.translate_historic_user(update_user),
                    added: mode.added,
                    removed: mode.removed,
                    changed_by: self.translate_state_change_source(mode.changed_by),
                },
                event,
            );
        }
    }

    pub(super) fn user_quit(
        &mut self,
        target: UserId,
        event: &Event,
        quit: &details::UserQuit,
        updates: &dyn NetworkUpdateReceiver,
    ) {
        if let Some(update) = self.remove_user(target, quit.message.clone()) {
            updates.notify(update, event);
        }
    }

    pub(super) fn enable_persistent_session(
        &mut self,
        target: UserId,
        event: &Event,
        detail: &details::EnablePersistentSession,
        _updates: &dyn NetworkUpdateReceiver,
    ) {
        if let Some(user) = self.users.get_mut(&target) {
            // If there's an existing key, then do conflict resolution
            if let Some(session_key) = &user.session_key {
                // Newest wins - users should be able to regenerate keys if needed
                if session_key.timestamp > event.timestamp {
                    return;
                }
                // If the TSes are the same, then do the usual lexicographical event ID comparison to tiebreak
                if session_key.timestamp == event.timestamp && session_key.enabled_by < event.id {
                    return;
                }
            }

            // If we get here, then we should update
            user.session_key = Some(UserSessionKey {
                timestamp: event.timestamp,
                enabled_by: event.id,
                key_hash: detail.key_hash.clone(),
            });
        }
    }
}
