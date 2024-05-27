//! Defines errors returned by the other modules

use crate::prelude::*;
use thiserror::Error;

/// Types of error that can occur while looking up network objects
#[derive(Error, Debug)]
pub enum LookupError {
    #[error("Wrong ID type")]
    WrongType,
    #[error("No such user id {0:?}")]
    NoSuchUser(UserId),
    #[error("No such historic user id {0:?}")]
    NoSuchHistoricUser(HistoricUserId),
    #[error("No such connection id {0:?}")]
    NoSuchConnection(UserConnectionId),
    #[error("No such channel id {0:?}")]
    NoSuchChannel(ChannelId),
    #[error("No such banlist id {0:?}")]
    NoSuchListMode(ListModeId),
    #[error("No channel corresponds to banlist id {0:?}")]
    NoChannelForList(ListModeId),
    #[error("No such channel topic id {0:?}")]
    NoSuchChannelTopic(ChannelTopicId),
    #[error("No topic for channel id {0:?}")]
    NoTopicForChannel(ChannelId),
    #[error("No such membership id {0:?}")]
    NoSuchMembership(MembershipId),
    #[error("No such invite id {0:?}")]
    NoSuchInvite(InviteId),
    #[error("No such server {0:?}")]
    NoSuchServer(ServerId),
    #[error("No such nickname {0}")]
    NoSuchNick(String),
    #[error("Couldn't find nickname for user {0:?}")]
    NoNickForUser(UserId),
    #[error("No such channel name {0}")]
    NoSuchChannelName(ChannelName),
    #[error("No such message id {0:?}")]
    NoSuchMessage(MessageId),
    #[error("Connection id not found")]
    NoSuchConnectionId,
    #[error("No such audit log entry {0:?}")]
    NoSuchAuditLogEntry(AuditLogEntryId),
    #[error("No such account {0:?}")]
    NoSuchAccount(AccountId),
    #[error("No such account named {0:?}")]
    NoSuchAccountNamed(Nickname),
    #[error("No such nick registration {0:?}")]
    NoSuchNickRegistration(NickRegistrationId),
    #[error("No registration for nick {0:?}")]
    NickNotRegistered(Nickname),
    #[error("No such channel registration {0:?}")]
    NoSuchChannelRegistration(ChannelRegistrationId),
    #[error("No registration for channel {0:?}")]
    ChannelNotRegistered(ChannelName),
    #[error("No such channel access {0:?}")]
    NoSuchChannelAccess(ChannelAccessId),
    #[error("No such channel role {0:?}")]
    NoSuchChannelRole(ChannelRoleId),
}

/// Convenience definition of a Result type used to look up network objects.
pub type LookupResult<T> = std::result::Result<T, LookupError>;
