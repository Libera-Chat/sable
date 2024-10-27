use crate::{
    history::{HistoricalEvent, HistoryError, HistoryRequest},
    id::*,
    network::{event::*, state::ChannelAccessSet, Network},
    validated::*,
};
use tokio::sync::{mpsc::Sender, oneshot};

/// A message emitted from the `ircd_sync` component when something
/// needs to be handled by the server logic.
#[derive(Debug)]
// The largest variant is NewEvent, which is the most commonly constructed one
#[allow(clippy::large_enum_variant)]
pub enum NetworkMessage {
    /// An export of the current network state is required. A clone
    /// of the [Network] object should be sent across the provided channel.
    ExportNetworkState(Sender<Box<Network>>),

    /// A serialised network state has been received from the network,
    /// and should be loaded into the server's view of state.
    ImportNetworkState(Box<Network>),

    /// An event has been propagated through the network, and should be
    /// applied to the server's view of state.
    NewEvent(Event),

    /// A message to be handled by a remote node
    RemoteServerRequest(RemoteServerRequest),
}

#[derive(Debug)]
pub struct RemoteServerRequest {
    pub req: RemoteServerRequestType,
    pub response: oneshot::Sender<RemoteServerResponse>,
}

/// A message to be handled by a specific node
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RemoteServerRequestType {
    /// Simple ping for communication tests
    Ping,
    /// A message to be handled by a services node
    Services(RemoteServicesServerRequestType),
    /// A message to be handled by a history node
    History(RemoteHistoryServerRequestType),
}

impl From<RemoteServicesServerRequestType> for RemoteServerRequestType {
    fn from(req: RemoteServicesServerRequestType) -> Self {
        RemoteServerRequestType::Services(req)
    }
}

impl From<RemoteHistoryServerRequestType> for RemoteServerRequestType {
    fn from(req: RemoteHistoryServerRequestType) -> Self {
        RemoteServerRequestType::History(req)
    }
}

/// A message to be handled by a services node
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RemoteServicesServerRequestType {
    /// User attempting registration
    /// Parameters: account name being registered, password provided
    RegisterUser(Nickname, String),
    /// User attempting login
    /// Parameters: account id, password
    UserLogin(AccountId, String),
    /// Begin SASL auth
    BeginAuthenticate(SaslSessionId, String),
    /// SASL traffic
    Authenticate(SaslSessionId, Vec<u8>),
    /// Abort a SASL session
    AbortAuthenticate(SaslSessionId),
    /// Register a channel
    RegisterChannel(AccountId, ChannelId),
    /// Add, modify or remove a channel access (None to delete)
    ModifyAccess {
        source: AccountId,
        id: ChannelAccessId,
        role: Option<ChannelRoleId>,
    },
    /// Create a channel role
    CreateRole {
        source: AccountId,
        channel: ChannelRegistrationId,
        name: CustomRoleName,
        flags: ChannelAccessSet,
    },
    /// Modify or delete a channel role
    ModifyRole {
        source: AccountId,
        id: ChannelRoleId,
        flags: Option<ChannelAccessSet>,
    },
    /// Add an authorised fingerprint to an account
    AddAccountFingerprint(AccountId, String),
    /// Remove an authorised fingerprint from an account
    RemoveAccountFingerprint(AccountId, String),
}

/// A message to be handled by a services node
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RemoteHistoryServerRequestType {
    ListTargets {
        user: UserId,
        after_ts: Option<i64>,
        before_ts: Option<i64>,
        limit: Option<usize>,
    },

    GetEntries {
        user: UserId,
        target: crate::history::TargetId,
        request: HistoryRequest,
    },
}

/// A SASL authentication response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AuthenticateStatus {
    /// Authentication flow should continue, with the enclosed data to be sent to the client
    InProgress(Vec<u8>),
    /// Authentication success; the user should be logged in to the enclosed account ID
    Success(AccountId),
    /// Authentication failed
    Fail,
    /// Authentication aborted
    Aborted,
}

/// Remote server response type
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RemoteServerResponse {
    /// Operation succeeded, no output
    Success,
    /// Operation not supported by this server
    NotSupported,
    /// Operation failed, with error message
    Error(String),
    /// Response type specific to services servers
    Services(RemoteServicesServerResponse),
    /// Response type specific to history servers
    History(RemoteHistoryServerResponse),
}

impl From<RemoteServicesServerResponse> for RemoteServerResponse {
    fn from(resp: RemoteServicesServerResponse) -> Self {
        RemoteServerResponse::Services(resp)
    }
}

impl From<RemoteHistoryServerResponse> for RemoteServerResponse {
    fn from(resp: RemoteHistoryServerResponse) -> Self {
        RemoteServerResponse::History(resp)
    }
}

/// Remote services server response type
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RemoteServicesServerResponse {
    /// Operation succeeded, user should be logged in to account
    LogUserIn(AccountId),
    /// SASL response
    Authenticate(AuthenticateStatus),
    /// Operation failed due to invalid credentials
    InvalidCredentials,
    /// Registration failed because the account exists
    AlreadyExists,
    /// Operation failed because of insufficient privileges
    AccessDenied,
    /// User isn't registered or account doesn't exist
    NoAccount,
    /// Channel isn't registered
    ChannelNotRegistered,
}

/// Remote history server response type
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RemoteHistoryServerResponse {
    /// TODO: switch to HashMap when we move away from JSON as the wire format,
    /// to be consistent with [`HistoryService`]
    TargetList(Vec<(crate::history::TargetId, i64)>),
    Entries(Result<Vec<HistoricalEvent>, HistoryError>),
}
