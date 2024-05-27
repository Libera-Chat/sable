use super::*;
use crate::messages::OutboundMessageTag;
use sable_network::{network::Network, prelude::NetworkStateChange};

fn account_for_tag(update: &NetworkStateChange, net: &Network) -> Option<String> {
    let id = match update {
        NetworkStateChange::UserNickChange(detail) => Some(&detail.user),
        NetworkStateChange::UserAwayChange(detail) => Some(&detail.user),
        NetworkStateChange::UserQuit(detail) => Some(&detail.user),
        NetworkStateChange::MembershipFlagChange(detail) => Some(&detail.user),
        NetworkStateChange::ChannelJoin(detail) => Some(&detail.user),
        NetworkStateChange::ChannelPart(detail) => Some(&detail.user),
        NetworkStateChange::UserLoginChange(detail) => Some(&detail.user),

        NetworkStateChange::ChannelRename(detail) => detail.source.user(),
        NetworkStateChange::ChannelInvite(detail) => detail.source.user(),
        NetworkStateChange::NewMessage(detail) => detail.source.user(),
        NetworkStateChange::ChannelKick(detail) => detail.source.user(),
        NetworkStateChange::ChannelModeChange(detail) => detail.changed_by.user(),
        NetworkStateChange::ChannelTopicChange(detail) => detail.setter.user(),
        NetworkStateChange::ListModeAdded(detail) => detail.set_by.user(),
        NetworkStateChange::ListModeRemoved(detail) => detail.removed_by.user(),
        NetworkStateChange::NewUser(_) => None,
        NetworkStateChange::NewUserConnection(_) => None,
        NetworkStateChange::UserConnectionDisconnected(_) => None,
        NetworkStateChange::UserModeChange(_) => None,
        NetworkStateChange::NewServer(_) => None,
        NetworkStateChange::ServerQuit(_) => None,
        NetworkStateChange::NewAuditLogEntry(_) => None,
        NetworkStateChange::ServicesUpdate(_) => None,
        NetworkStateChange::EventComplete(_) => None,
    }?;
    Some(net.historic_user(*id).ok()?.account?.to_string())
}

pub fn account_tag(update: &NetworkStateChange, net: &Network) -> Option<OutboundMessageTag> {
    account_for_tag(update, net).map(|account| {
        OutboundMessageTag::new("account", Some(account), ClientCapability::AccountTag)
    })
}
