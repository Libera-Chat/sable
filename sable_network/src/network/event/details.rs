use crate::prelude::*;

use sable_macros::{event_details, target_type};

pub struct WrongEventTypeError;

event_details!(

/// Defines the type and details of an [`Event`](event::Event).
EventDetails => {
    #[target_type(NicknameId)]
    struct BindNickname {
        pub user: UserId,
    }

    #[target_type(UserId)]
    struct NewUser {
        pub nickname: Nickname,
        pub username: Username,
        pub visible_hostname: Hostname,
        pub realname: String,
        pub mode: state::UserMode,
        pub server: ServerId,
        pub account: Option<AccountId>,
    }

    #[target_type(UserId)]
    struct UserQuit {
        pub message: String,
    }

    #[target_type(UserId)]
    struct UserModeChange {
        pub changed_by: ObjectId,
        pub added: UserModeSet,
        pub removed: UserModeSet,
    }

    #[target_type(UserId)]
    struct OperUp {
        pub oper_name: String
    }

    #[target_type(ChannelId)]
    struct NewChannel {
        pub name: ChannelName,
        pub mode: state::ChannelMode,
    }

    #[target_type(ChannelId)]
    struct ChannelModeChange {
        pub changed_by: ObjectId,
        pub added: ChannelModeSet,
        pub removed: ChannelModeSet,
        pub key_change: OptionChange<ChannelKey>,
    }

    #[target_type(ListModeEntryId)]
    struct NewListModeEntry {
        pub list: ListModeId,
        pub pattern: Pattern,
        pub setter: UserId,
    }

    #[target_type(ListModeEntryId)]
    struct DelListModeEntry {
        pub removed_by: UserId,
    }

    #[target_type(ChannelTopicId)]
    struct NewChannelTopic {
        pub channel: ChannelId,
        pub text: String,
        pub setter: ObjectId,
    }

    #[target_type(MembershipId)]
    struct ChannelJoin {
        pub channel: ChannelId,
        pub user: UserId,
        pub permissions: MembershipFlagSet,
    }

    #[target_type(MembershipId)]
    struct MembershipFlagChange {
        pub changed_by: ObjectId,
        pub added: MembershipFlagSet,
        pub removed: MembershipFlagSet,
    }

    #[target_type(MembershipId)]
    struct ChannelPart {
        pub message: String,
    }

    #[target_type(InviteId)]
    struct ChannelInvite {
        pub source: UserId,
    }

    #[target_type(MessageId)]
    struct NewMessage {
        pub source: UserId,
        pub target: ObjectId, // Can be user or channel
        pub message_type: state::MessageType,
        pub text: String,
    }

    #[target_type(NetworkBanId)]
    struct NewNetworkBan {
        pub matcher: ban::NetworkBanMatch,
        pub action: ban::NetworkBanAction,

        pub timestamp: i64,
        pub expires: i64,
        pub reason: String,
        pub oper_reason: Option<String>,
        pub setter_info: String,

    }

    #[target_type(NetworkBanId)]
    struct RemoveNetworkBan {
        pub remover: UserId,
    }

    #[target_type(ServerId)]
    struct NewServer {
        pub epoch: EpochId,
        pub name: ServerName,
        pub ts: i64,
        pub flags: state::ServerFlags,
        pub version: String,
    }

    #[target_type(ServerId)]
    struct ServerPing {
        pub ts: i64,
    }

    #[target_type(ServerId)]
    struct ServerQuit {
        pub epoch: EpochId,
    }

    #[target_type(ConfigId)]
    struct LoadConfig {
        pub config: config::NetworkConfig,
    }

    #[target_type(AuditLogEntryId)]
    struct NewAuditLogEntry {
        pub entry: state::AuditLogEntry,
    }

    #[target_type(UserId)]
    struct EnablePersistentSession {
        pub key_hash: String
    }

    #[target_type(ServerId)]
    struct IntroduceServices {
        pub sasl_mechanisms: Vec<String>,
    }

    #[target_type(AccountId)]
    struct AccountUpdate {
        pub data: Option<state::Account>,
    }

    #[target_type(NickRegistrationId)]
    struct NickRegistrationUpdate {
        pub data: Option<state::NickRegistration>,
    }

    #[target_type(ChannelRegistrationId)]
    struct ChannelRegistrationUpdate {
        pub data: Option<state::ChannelRegistration>,
    }

    #[target_type(ChannelAccessId)]
    struct ChannelAccessUpdate {
        pub data: Option<state::ChannelAccess>,
    }

    #[target_type(ChannelRoleId)]
    struct ChannelRoleUpdate {
        pub data: Option<state::ChannelRole>,
    }

    #[target_type(UserId)]
    struct UserAway {
        /// Empty iff the user is back from being away
        pub reason: String,
    }

    #[target_type(UserId)]
    struct UserLogin {
        /// None means logout
        pub account: Option<AccountId>
    }
});
