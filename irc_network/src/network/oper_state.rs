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

            if let Some(mode) = self.user_modes.get_mut(&user.mode_id)
            {
                mode.modes |= UserModeFlag::Oper;

                if new_oper
                {
                    updates.notify(update::UserModeChange {
                        user_id: target,
                        mode_id: user.mode_id,
                        added: UserModeFlag::Oper.into(),
                        removed: UserModeSet::new(),
                        changed_by: target.into()
                    });
                }
            }
        }
    }
}