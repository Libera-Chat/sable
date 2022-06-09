use super::*;
use crate::update::*;
use crate::state::*;

impl Network
{
    pub(super) fn oper_up(&mut self, target: UserId, _event: &Event, details: &details::OperUp, updates: &dyn NetworkUpdateReceiver)
    {
        if let Some(user) = self.users.get_mut(&target)
        {
            let new_oper = user.oper_privileges.is_none();

            user.oper_privileges = Some(UserPrivileges {
                oper_name: details.oper_name.clone()
            });

            user.mode.modes |= UserModeFlag::Oper;

            if new_oper
            {
                let update_user = user.clone();

                updates.notify(update::UserModeChange {
                    user: self.translate_historic_user(update_user),
                    added: UserModeFlag::Oper.into(),
                    removed: UserModeSet::new(),
                    changed_by: self.translate_state_change_source(target.into())
                });
            }
        }
    }
}