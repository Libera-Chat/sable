use crate::network::state::ChannelAccessFlag;

use super::*;

/// Standard implementation of [`ChannelPolicyService`]
pub struct StandardRegistrationPolicy;

impl StandardRegistrationPolicy
{
    pub fn new() -> Self
    {
        Self
    }
}

impl RegistrationPolicyService for StandardRegistrationPolicy
{
    fn can_view_access(&self, source: &wrapper::User, channel: &wrapper::ChannelRegistration) -> PermissionResult
    {
        let source_account = source.account()?
                                   .ok_or(RegistrationPermissionError::NotLoggedIn)?;

        let source_access = source_account.has_access_in(channel.id())
                                          .ok_or(RegistrationPermissionError::NoAccess)?;

        if ! source_access.role()?.flags().is_set(ChannelAccessFlag::AccessView)
        {
            return Err(RegistrationPermissionError::NoAccess.into());
        }

        Ok(())
    }

    fn can_view_roles(&self,source: &wrapper::User,channel: &wrapper::ChannelRegistration) -> PermissionResult
    {
        let source_account = source.account()?
                                   .ok_or(RegistrationPermissionError::NotLoggedIn)?;

        let source_access = source_account.has_access_in(channel.id())
                                          .ok_or(RegistrationPermissionError::NoAccess)?;

        if ! source_access.role()?.flags().is_set(ChannelAccessFlag::RoleView)
        {
            return Err(RegistrationPermissionError::NoAccess.into());
        }

        Ok(())
    }

    fn can_change_access_for(&self, source: &wrapper::Account, chan: &wrapper::ChannelRegistration, target: &wrapper::Account) -> PermissionResult
    {
        let source_access = source.has_access_in(chan.id())
                                  .ok_or(RegistrationPermissionError::NoAccess)?;

        if ! source_access.role()?.flags().is_set(ChannelAccessFlag::AccessEdit)
        {
            return Err(RegistrationPermissionError::NoAccess.into());
        }

        let target_access = target.has_access_in(chan.id());

        if let Some(current_flags) = target_access.and_then(|access| access.role().ok().map(|r| r.flags()))
        {
            if ! source_access.role()?.flags().dominates(&current_flags)
            {
                return Err(RegistrationPermissionError::CantEditHigherRole.into());
            }
        }

        Ok(())

    }

    fn can_grant_role(&self, source: &wrapper::Account, channel: &wrapper::ChannelRegistration, role: &wrapper::ChannelRole) -> PermissionResult
    {
        let source_access = source.has_access_in(channel.id())
                                  .ok_or(RegistrationPermissionError::NoAccess)?;

        if ! source_access.role()?.flags().is_set(ChannelAccessFlag::AccessEdit)
        {
            return Err(RegistrationPermissionError::NoAccess.into());
        }

        if ! source_access.role()?.flags().dominates(&role.flags())
        {
            return Err(RegistrationPermissionError::CantEditHigherRole.into());
        }

        Ok(())
    }

    fn can_edit_role(&self,source: &wrapper::Account,channel: &wrapper::ChannelRegistration,role: &wrapper::ChannelRole) -> PermissionResult
    {
        let source_access = source.has_access_in(channel.id())
                                  .ok_or(RegistrationPermissionError::NoAccess)?;

        if ! source_access.role()?.flags().is_set(ChannelAccessFlag::RoleEdit)
        {
            return Err(RegistrationPermissionError::NoAccess.into());
        }

        if ! source_access.role()?.flags().dominates(&role.flags())
        {
            return Err(RegistrationPermissionError::CantEditHigherRole.into());
        }

        Ok(())
    }

    fn can_create_role(&self,source: &wrapper::Account,channel: &wrapper::ChannelRegistration,with_flags: &state::ChannelAccessSet) -> PermissionResult
    {
        let source_access = source.has_access_in(channel.id())
                                  .ok_or(RegistrationPermissionError::NoAccess)?;

        if ! source_access.role()?.flags().is_set(ChannelAccessFlag::RoleEdit)
        {
            return Err(RegistrationPermissionError::NoAccess.into());
        }

        if ! source_access.role()?.flags().dominates(with_flags)
        {
            return Err(RegistrationPermissionError::CantEditHigherRole.into());
        }

        Ok(())
    }
}