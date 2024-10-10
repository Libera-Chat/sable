use super::*;

use rpc::NetworkHistoryUpdate;

impl HistoryServer {
    pub fn handle_history_update(&self, update: NetworkHistoryUpdate) {
        match update.change {
            NetworkStateChange::NewUser(new_user) => todo!(),
            NetworkStateChange::UserNickChange(user_nick_change) => todo!(),
            NetworkStateChange::UserModeChange(user_mode_change) => todo!(),
            NetworkStateChange::UserAwayChange(user_away_change) => todo!(),
            NetworkStateChange::UserQuit(user_quit) => todo!(),
            NetworkStateChange::NewUserConnection(new_user_connection) => todo!(),
            NetworkStateChange::UserConnectionDisconnected(user_connection_disconnected) => todo!(),
            NetworkStateChange::ChannelModeChange(channel_mode_change) => todo!(),
            NetworkStateChange::ChannelTopicChange(channel_topic_change) => todo!(),
            NetworkStateChange::ListModeAdded(list_mode_added) => todo!(),
            NetworkStateChange::ListModeRemoved(list_mode_removed) => todo!(),
            NetworkStateChange::MembershipFlagChange(membership_flag_change) => todo!(),
            NetworkStateChange::ChannelJoin(channel_join) => todo!(),
            NetworkStateChange::ChannelKick(channel_kick) => todo!(),
            NetworkStateChange::ChannelPart(channel_part) => todo!(),
            NetworkStateChange::ChannelInvite(channel_invite) => todo!(),
            NetworkStateChange::ChannelRename(channel_rename) => todo!(),
            NetworkStateChange::NewMessage(new_message) => todo!(),
            NetworkStateChange::NewServer(new_server) => todo!(),
            NetworkStateChange::ServerQuit(server_quit) => todo!(),
            NetworkStateChange::NewAuditLogEntry(new_audit_log_entry) => todo!(),
            NetworkStateChange::UserLoginChange(user_login_change) => todo!(),
            NetworkStateChange::ServicesUpdate(services_update) => todo!(),
            NetworkStateChange::EventComplete(event_complete) => todo!(),
        }
    }
}
