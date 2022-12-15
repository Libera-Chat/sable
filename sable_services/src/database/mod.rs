use sable_network::prelude::*;
use crate::model::*;

use thiserror::Error;

#[derive(Debug,Error)]
pub enum DatabaseError
{
    #[error("Duplicate object ID")]
    DuplicateId,
    #[error("Duplicate object name")]
    DuplicateName,
    #[error("No such object ID")]
    NoSuchId,
    #[error("Invalid data")]
    InvalidData,
    #[error("{0}")]
    DbError(#[from] Box<dyn std::error::Error + 'static>)
}

impl DatabaseError
{
    fn from_inner<T: std::error::Error + 'static>(inner: T) -> Self
    {
        Self::DbError(Box::new(inner))
    }
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

/// Trait defining a database provider
///
/// Note that creation functions take their argument by value and return it; callers
/// must use the returned value for any further use or propagation to the network as
/// content, including IDs, may have been changed by the database.
pub trait DatabaseConnection : Sized
{
    /// Constructor. The format of `conn` is defined by the provider and taken from the
    /// server config file.
    fn connect(conn: impl AsRef<str>) -> Result<Self>;

    /// Create a new account, store it in the database, and return it
    fn new_account(&self, data: state::Account, auth: AccountAuth) -> Result<state::Account>;
    /// Retrieve a single account
    fn account(&self, id: AccountId) -> Result<state::Account>;
    /// Update an account's details
    fn update_account(&self, new_data: &state::Account) -> Result<()>;
    /// Retrieve all accounts in the database
    fn all_accounts(&self) -> Result<impl Iterator<Item=state::Account> + '_>;

    /// Retrieve the authentication data for a given account
    fn auth_for_account(&self, id: AccountId) -> Result<AccountAuth>;

    /// Create a new nick registration, store it in the database, and return it
    fn new_nick_registration(&self, data: state::NickRegistration) -> Result<state::NickRegistration>;
    /// Retrieve a single nick registration
    fn nick_registration(&self, id: NickRegistrationId) -> Result<state::NickRegistration>;
    /// Update a nick registration
    fn update_nick_registration(&self, new_data: &state::NickRegistration) -> Result<()>;
    /// Retrieve all nick registrations in the database
    fn all_nick_registrations(&self) -> Result<impl Iterator<Item=state::NickRegistration> + '_>;

    /// Create a new channel registration, store it in the database, and return it
    fn new_channel_registration(&self, data: state::ChannelRegistration) -> Result<state::ChannelRegistration>;
    /// Retrieve a single channel registration
    fn channel_registration(&self, id: ChannelRegistrationId) -> Result<state::ChannelRegistration>;
    /// Update a channel registration
    fn update_channel_registration(&self, new_data: &state::ChannelRegistration) -> Result<()>;
    /// Retrieve all channel registrations in the database
    fn all_channel_registrations(&self) -> Result<impl Iterator<Item=state::ChannelRegistration> + '_>;

    /// Create a new channel role
    fn new_channel_role(&self, data: state::ChannelRole) -> Result<state::ChannelRole>;
    /// Retrieve a channel role
    fn channel_role(&self, id: ChannelRoleId) -> Result<state::ChannelRole>;
    /// Update a channel role
    fn update_channel_role(&self, data: &state::ChannelRole) -> Result<()>;
    /// Retrieve all channel roles in the database
    fn all_channel_roles(&self) -> Result<impl Iterator<Item=state::ChannelRole> + '_>;

    /// Create or update a channel access
    /// Unlike other creation functions, this one doesn't return the state object because
    /// the ID is keyed to other objects and can't be changed by the DB layer
    fn update_channel_access(&self, data: &state::ChannelAccess) -> Result<()>;
    /// Retrieve a single channel access
    fn channel_access(&self, id: ChannelAccessId) -> Result<state::ChannelAccess>;
    /// Retrieve all channel accesses in the database
    fn all_channel_accesses(&self) -> Result<impl Iterator<Item=state::ChannelAccess> + '_>;
    /// Remove a channel access
    fn remove_channel_access(&self, id: ChannelAccessId) -> Result<()>;
}

pub mod jsonfile;