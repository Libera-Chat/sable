use crate::{
    network::{
        event::*,
        Network,
        state::ChannelAccessSet,
    },
    id::*,
    validated::*,
};
use tokio::sync::{
    mpsc::Sender,
    oneshot,
};

/// A message emitted from the `ircd_sync` component when something
/// needs to be handled by the server logic.
#[derive(Debug)]
// The largest variant is NewEvent, which is the most commonly constructed one
#[allow(clippy::large_enum_variant)]
pub enum NetworkMessage
{
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
pub struct RemoteServerRequest
{
    pub req: RemoteServerRequestType,
    pub response: oneshot::Sender<RemoteServerResponse>
}

/// A message to be handled by a services node
#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub enum RemoteServerRequestType
{
    /// Simple ping for communication tests
    Ping,
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
    ModifyAccess{ source: AccountId, id: ChannelAccessId, role: Option<ChannelRoleId> },
    /// Create a channel role
    CreateRole{ source: AccountId, channel: ChannelRegistrationId, name: CustomRoleName, flags: ChannelAccessSet },
    /// Modify or delete a channel role
    ModifyRole{ source: AccountId, id: ChannelRoleId, flags: Option<ChannelAccessSet> },
}

/// A SASL authentication response
#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub enum AuthenticateStatus
{
    /// Authentication flow should continue, with the enclosed data to be sent to the client
    InProgress(Vec<u8>),
    /// Authentication success; the user should be logged in to the enclosed account ID
    Success(AccountId),
    /// Authentication failed
    Fail,
    /// Authentication aborted
    Aborted
}

/// Remote server response type
#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub enum RemoteServerResponse
{
    /// Operation succeeded, no output
    Success,
    /// Operation not supported by this server
    NotSupported,
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
    /// Operation failed, with error message
    Error(String),
}

