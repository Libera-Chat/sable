//! Defines the [Network] object.

use crate::network::event::*;
use crate::network::update::*;
use crate::prelude::*;

use sable_macros::dispatch_event;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use thiserror::Error;

use std::collections::HashMap;
use std::convert::TryInto;

use once_cell::sync::OnceCell;

/// Error enumeration defining possible problems to be returned from
/// the [Network::validate] method.
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Nickname {0} already in use")]
    NickInUse(Nickname),
    #[error("Object not found: {0}")]
    ObjectNotFound(#[from] LookupError),
    #[error("Wrong object ID type: {0}")]
    WrongTypeId(#[from] WrongIdTypeError),
    #[error("{0}")]
    InvalidNickname(#[from] InvalidNicknameError),
    #[error("{0}")]
    InvalidUsername(#[from] InvalidUsernameError),
    #[error("{0}")]
    InvalidHostname(#[from] InvalidHostnameError),
    #[error("{0}")]
    InvalidChannelName(#[from] InvalidChannelNameError),
}

/// Convenience definition for a Result whose Error type is ValidationError
pub type ValidationResult = Result<(), ValidationError>;

/// Stores the current network state.
///
/// ## General Principles
///
/// A `Network` object is fully serializable and cloneable;
/// all objects within it refer to each other by unique ID
/// and not by reference.
///
/// The `Network` stores only raw state objects, which themselves provide no
/// logic or other utility. Short-lived wrapper objects are created and
/// returned by most public methods, which wrap a reference to the underlying
/// state and provide convenience accessors for associated objects and various
/// other pieces of application logic.
///
/// In line with Rust's borrowing rules, these wrappers cannot outlive the
/// calling code's borrow of the `Network`, and should not be stored. If a list
/// of network objects needs to be maintained by code outside of this module,
/// then it should store object IDs and look them up as required.
///
/// Most public accessors return a [`LookupResult`] instead of an `Option` to
/// facilitate handling of missing objects in command handlers.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    // All of these maps are serialised as an array of tuples
    // because their keys don't serialise as strings, so can't be
    // used as a JSON object key.
    #[serde_as(as = "Vec<(_,_)>")]
    nick_bindings: HashMap<Nickname, state::NickBinding>,
    #[serde_as(as = "Vec<(_,_)>")]
    users: HashMap<UserId, state::User>,

    #[serde_as(as = "Vec<(_,_)>")]
    channels: HashMap<ChannelId, state::Channel>,
    #[serde_as(as = "Vec<(_,_)>")]
    list_mode_entries: HashMap<ListModeEntryId, state::ListModeEntry>,
    #[serde_as(as = "Vec<(_,_)>")]
    channel_topics: HashMap<ChannelTopicId, state::ChannelTopic>,
    #[serde_as(as = "Vec<(_,_)>")]
    channel_invites: HashMap<InviteId, state::ChannelInvite>,

    #[serde_as(as = "Vec<(_,_)>")]
    memberships: HashMap<MembershipId, state::Membership>,

    #[serde_as(as = "Vec<(_,_)>")]
    messages: HashMap<MessageId, state::Message>,

    #[serde_as(as = "Vec<(_,_)>")]
    servers: HashMap<ServerId, state::Server>,

    network_bans: ban::BanRepository,

    #[serde_as(as = "Vec<(_,_)>")]
    audit_log: HashMap<AuditLogEntryId, state::AuditLogEntry>,

    #[serde_as(as = "Vec<(_,_)>")]
    accounts: HashMap<AccountId, state::Account>,

    #[serde_as(as = "Vec<(_,_)>")]
    nick_registrations: HashMap<NickRegistrationId, state::NickRegistration>,

    #[serde_as(as = "Vec<(_,_)>")]
    channel_registrations: HashMap<ChannelRegistrationId, state::ChannelRegistration>,

    #[serde_as(as = "Vec<(_,_)>")]
    channel_accesses: HashMap<ChannelAccessId, state::ChannelAccess>,

    #[serde_as(as = "Vec<(_,_)>")]
    channel_roles: HashMap<ChannelRoleId, state::ChannelRole>,

    current_services: Option<state::ServicesData>,
    config: config::NetworkConfig,

    clock: EventClock,

    // Cached or constructed data that doesn't need to be serialised
    #[serde(skip)]
    cache_default_channel_roles: OnceCell<HashMap<state::ChannelRoleName, state::ChannelRole>>,

    #[serde(skip)]
    alias_users: OnceCell<HashMap<Nickname, state::User>>,
}

impl Network {
    /// Create an empty network state.
    pub fn new(config: config::NetworkConfig) -> Network {
        let net = Network {
            nick_bindings: HashMap::new(),
            users: HashMap::new(),

            channels: HashMap::new(),
            channel_topics: HashMap::new(),
            list_mode_entries: HashMap::new(),
            memberships: HashMap::new(),
            channel_invites: HashMap::new(),

            messages: HashMap::new(),
            servers: HashMap::new(),
            network_bans: ban::BanRepository::new(),

            audit_log: HashMap::new(),

            accounts: HashMap::new(),
            nick_registrations: HashMap::new(),
            channel_registrations: HashMap::new(),
            channel_accesses: HashMap::new(),
            channel_roles: HashMap::new(),

            current_services: None,
            config,

            clock: EventClock::new(),

            cache_default_channel_roles: OnceCell::new(),
            alias_users: OnceCell::new(),
        };

        net.build_default_role_cache();
        net.build_alias_users();
        net
    }

    /// Apply an [Event] to the network state.
    ///
    /// This is the only supported way to update the state. Events should
    /// be applied as they are emitted by the event log.
    ///
    /// ## Arguments
    ///
    /// - `event`: the event to apply
    /// - `updates`: an implementation of [NetworkUpdateReceiver] which will
    ///   be used to notify the caller of any changes in network state that result
    ///   from the processing of this event.
    ///
    /// ## Return Value
    ///
    /// `Ok(())` if the event was successfully applied. `Err(_)` if there is a
    /// mismatch between the expected target object for the event type and the
    /// provided target ID type.
    ///
    /// This function is infallible if a properly-formed `Event` is supplied.
    ///
    /// ## Side Effects
    ///
    /// - The network state is updated to reflect the application of the event
    /// - The network's event clock is updated to reflect the incoming event ID.
    /// - The `notify_update` method is called zero or more times on `updates`
    ///
    pub fn apply(
        &mut self,
        event: &Event,
        updates: &dyn NetworkUpdateReceiver,
    ) -> Result<(), WrongIdTypeError> {
        if self.clock.contains(event.id) {
            return Ok(());
        }

        dispatch_event!(event(updates) => {
            BindNickname => self.bind_nickname,
            NewUser => self.new_user,
            UserQuit => self.user_quit,
            UserModeChange => self.user_mode_change,
            OperUp => self.oper_up,
            NewChannel => self.new_channel,
            ChannelModeChange => self.channel_mode_change,
            NewListModeEntry => self.new_list_mode_entry,
            DelListModeEntry => self.del_list_mode_entry,
            NewChannelTopic => self.new_channel_topic,
            MembershipFlagChange => self.channel_permission_change,
            ChannelJoin => self.user_joined_channel,
            ChannelPart => self.user_left_channel,
            ChannelInvite => self.new_channel_invite,
            NewMessage => self.new_message,
            NewNetworkBan => self.new_ban,
            RemoveNetworkBan => self.remove_ban,
            NewServer => self.new_server,
            ServerPing => self.server_ping,
            ServerQuit => self.server_quit,
            LoadConfig => self.load_config,
            NewAuditLogEntry => self.new_audit_log,
            EnablePersistentSession => self.enable_persistent_session,
            IntroduceServices => self.introduce_services,
            AccountUpdate => self.update_account,
            NickRegistrationUpdate => self.update_nick_registration,
            ChannelRegistrationUpdate => self.update_channel_registration,
            ChannelAccessUpdate => self.update_channel_access,
            ChannelRoleUpdate => self.update_channel_role,
            UserAway => self.user_away,
            UserLogin => self.user_login,
        })?;

        self.clock.update_with_id(event.id);
        updates.notify(EventComplete {}, &event);

        Ok(())
    }

    /// Validate a proposed event against the current network state.
    ///
    /// This method can be used to identify problems which would occur if the
    /// provided event details were transformed and applied to the current
    /// state. This is provided as a convenience for consumers, and does not
    /// give any guarantee of behaviour if other events are processed between
    /// the validation and eventual application of a given event.
    ///
    /// Note also that this is not related to whether [`apply`](Self::apply)
    ///  will succeed - `apply` always succeeds provided the event is well-
    /// formed. `validate`  provides advance warning of potentially undesirable
    /// effects, such as nickname collisions.
    pub fn validate(&self, id: ObjectId, detail: &EventDetails) -> ValidationResult {
        match detail {
            EventDetails::NewUser(newuser) => self.validate_new_user(id.try_into()?, newuser),
            _ => Ok(()),
        }
    }

    /// Translate an object ID into a [`update::HistoricMessageSource`]
    pub(crate) fn translate_state_change_source(
        &self,
        id: ObjectId,
    ) -> update::HistoricMessageSource {
        match id {
            ObjectId::User(user_id) => self.users.get(&user_id).map(|user| {
                update::HistoricMessageSource::User(self.translate_historic_user(user.clone()))
            }),
            ObjectId::Server(server_id) => self
                .servers
                .get(&server_id)
                .map(|s| update::HistoricMessageSource::Server(s.clone())),
            _ => None,
        }
        .unwrap_or(update::HistoricMessageSource::Unknown)
    }

    /// Translate a [`state::User`] to a [`HistoricUser`] based on the current network state
    pub(crate) fn translate_historic_user(&self, user: state::User) -> update::HistoricUser {
        let nickname = self.infallible_nick_for_user(user.id);
        update::HistoricUser { nickname, user }
    }

    /// Translate an [`ObjectId`] into a [`HistoricMessageTarget`] for storage in history log
    pub(crate) fn translate_message_target(&self, id: ObjectId) -> update::HistoricMessageTarget {
        match id {
            ObjectId::User(user_id) => self.users.get(&user_id).map(|user| {
                update::HistoricMessageTarget::User(self.translate_historic_user(user.clone()))
            }),
            ObjectId::Channel(channel_id) => self
                .channels
                .get(&channel_id)
                .map(|c| update::HistoricMessageTarget::Channel(c.clone())),
            _ => None,
        }
        .unwrap_or(update::HistoricMessageTarget::Unknown)
    }
}

mod accessors;
mod alias_users;
mod default_roles;

mod account_state;
mod audit_log;
mod ban_state;
mod channel_state;
mod config_state;
mod message_state;
mod oper_state;
mod server_state;
mod user_state;
