use super::*;
use client_listener::ConnectionError;

use crate::errors::HandlerError;

/// An error that may occur during command processing
///
/// Note that, at present, returning the `UnderlyingError` or `UnknownError` variants
/// from a handler will cause the default dispatcher to panic; in future this may
/// change (for example, to terminate the connection), but in either case should only
/// be used for exceptional, unhandleable, errors.
#[derive(Debug)]
pub enum CommandError {
    /// Something returned an `Error` that we don't know how to handle
    UnderlyingError(anyhow::Error),
    /// Something went wrong but we don't have an `Error` impl for it
    UnknownError(String),
    /*
        /// The command couldn't be processed successfully, and the client has already been
        /// notified
        CustomError,
    */
    /// The command wasn't recognised
    CommandNotFound(String),
    /// Not enough arguments were provided
    NotEnoughParameters,
    /// A required object wasn't found in the network state
    LookupError(LookupError),
    /// A nickname parameter wasn't a valid nick
    InvalidNick(String),
    /// A channel name parameter wasn't a valid channel name
    InvalidChannelName(String),
    /// A services command was executed, but services aren't currently running
    ServicesNotAvailable,
    /// The source user wasn't logged in and needs to be
    NotLoggedIn,
    /// The target channel isn't registered
    ChannelNotRegistered(ChannelName),
    /// A given parameter (.0) wasn't valid for the expected type (.1)
    InvalidArgument(String, String),

    /// A permission error was encountered
    Permission(policy::PermissionError),

    /// The command couldn't be processed successfully; the provided
    /// numeric(messages::UntargetedNumeric) will be sent to the client to notify them
    Numeric(messages::UntargetedNumeric),
}

impl From<ValidationError> for CommandError {
    fn from(e: ValidationError) -> Self {
        match e {
            ValidationError::NickInUse(n) => numeric::NicknameInUse::new(&n).into(),
            ValidationError::ObjectNotFound(le) => match le {
                LookupError::NoSuchNick(n) => numeric::NoSuchTarget::new(&n).into(),
                LookupError::NoSuchChannelName(n) => numeric::NoSuchTarget::new(n.as_ref()).into(),
                _ => CommandError::UnknownError(le.to_string()),
            },
            ValidationError::InvalidNickname(e) => numeric::ErroneousNickname::new(&e.0).into(),
            ValidationError::InvalidChannelName(e) => numeric::InvalidChannelName::new(&e.0).into(),
            ValidationError::InvalidUsername(e) => CommandError::UnknownError(e.0),
            ValidationError::InvalidHostname(e) => CommandError::UnknownError(e.0),
            ValidationError::WrongTypeId(e) => CommandError::UnknownError(e.to_string()),
        }
    }
}

impl From<policy::PermissionError> for CommandError {
    fn from(e: policy::PermissionError) -> Self {
        /*
                use policy::{
                    PermissionError::*,
                    UserPermissionError::*,
                    ChannelPermissionError::*,
                };

                match e
                {
                    User(NotOper) => numeric::NotOper::new().into(),
                    User(ReadOnlyUmode | NotLoggedIn) => Self::CustomError, // Setting or unsetting these umodes silently fails
                    Registration(_) => Self::CustomError, //
                    Channel(channel_name, channel_err) => {
                        match channel_err
                        {
                            UserNotOnChannel => numeric::UserNotOnChannel::new(&channel_name).into(),
                            NotOnChannel => numeric::NotOnChannel::new(&channel_name).into(),
                            UserOnChannel => numeric::OnChannel::new(&channel_name).into(),
                            UserNotOp => numeric::ChanOpPrivsNeeded::new(&channel_name).into(),
                            UserIsBanned => numeric::BannedOnChannel::new(&channel_name).into(),
                            CannotSendToChannel => numeric::CannotSendToChannel::new(&channel_name).into(),
                            InviteOnlyChannel => numeric::InviteOnlyChannel::new(&channel_name).into(),
                            BadChannelKey => numeric::BadChannelKey::new(&channel_name).into(),
                            NotRegistered | NoAccess => Self::CustomError,
                        }
                    },
                    InternalError(e) => Self::UnderlyingError(e)
                }
        */
        Self::Permission(e)
    }
}

impl CommandError {
    pub fn unknown(desc: impl std::fmt::Display) -> Self {
        Self::UnknownError(desc.to_string())
    }
    /*
        pub fn inner(err: impl std::error::Error + Clone + 'static) -> CommandError
        {
            CommandError::UnderlyingError(Box::new(err))
        }
    */
}

impl From<LookupError> for CommandError {
    fn from(e: LookupError) -> Self {
        Self::LookupError(e)
    }
}

impl From<InvalidNicknameError> for CommandError {
    fn from(e: InvalidNicknameError) -> Self {
        Self::InvalidNick(e.0)
    }
}

impl From<InvalidChannelNameError> for CommandError {
    fn from(e: InvalidChannelNameError) -> Self {
        Self::InvalidChannelName(e.0)
    }
}

impl From<UntargetedNumeric> for CommandError {
    fn from(n: UntargetedNumeric) -> Self {
        Self::Numeric(n)
    }
}

impl From<ConnectionError> for CommandError {
    fn from(e: ConnectionError) -> Self {
        Self::UnderlyingError(e.into())
    }
}

impl From<HandlerError> for CommandError {
    fn from(e: HandlerError) -> Self {
        Self::UnderlyingError(e.into())
    }
}
