use super::*;
use crate::messages::OutboundMessageTag;
use sable_network::prelude::NetworkStateChange;

fn account_for_tag(update: &NetworkStateChange) -> Option<String> {
    match update {
        NetworkStateChange::UserNickChange(detail) => detail.user.account,
        NetworkStateChange::UserAwayChange(detail) => detail.user.account,
        NetworkStateChange::UserQuit(detail) => detail.user.account,
        NetworkStateChange::MembershipFlagChange(detail) => detail.user.account,
        NetworkStateChange::ChannelJoin(detail) => detail.user.account,
        NetworkStateChange::ChannelPart(detail) => detail.user.account,
        NetworkStateChange::UserLoginChange(detail) => detail.user.account,

        NetworkStateChange::ChannelRename(detail) => detail.source.user().and_then(|u| u.account),
        NetworkStateChange::ChannelInvite(detail) => detail.source.user().and_then(|u| u.account),
        NetworkStateChange::NewMessage(detail) => detail.source.user().and_then(|u| u.account),
        NetworkStateChange::ChannelKick(detail) => detail.source.user().and_then(|u| u.account),
        NetworkStateChange::ChannelModeChange(detail) => {
            detail.changed_by.user().and_then(|u| u.account)
        }
        NetworkStateChange::ChannelTopicChange(detail) => {
            detail.setter.user().and_then(|u| u.account)
        }
        NetworkStateChange::ListModeAdded(detail) => detail.set_by.user().and_then(|u| u.account),
        NetworkStateChange::ListModeRemoved(detail) => {
            detail.removed_by.user().and_then(|u| u.account)
        }
        NetworkStateChange::NewUser(_) => None,
        NetworkStateChange::UserModeChange(_) => None,
        NetworkStateChange::BulkUserQuit(_) => None,
        NetworkStateChange::NewServer(_) => None,
        NetworkStateChange::ServerQuit(_) => None,
        NetworkStateChange::NewAuditLogEntry(_) => None,
        NetworkStateChange::ServicesUpdate(_) => None,
        NetworkStateChange::EventComplete(_) => None,
    }
    .map(|n| n.to_string())
}

pub fn account_tag(update: &NetworkStateChange) -> Option<OutboundMessageTag> {
    account_for_tag(update).map(|account| {
        OutboundMessageTag::new("account", Some(account), ClientCapability::AccountTag)
    })
}
