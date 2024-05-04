use crate::{client::PreClient, numeric_error};

use super::*;

pub struct UserSource<'a> {
    pub user: wrapper::User<'a>,
    pub user_connection: wrapper::UserConnection<'a>,
}

pub struct PreClientSource(pub Arc<PreClient>);
pub struct LoggedInUserSource<'a> {
    pub user: wrapper::User<'a>,
    pub user_connection: wrapper::UserConnection<'a>,
    pub account: wrapper::Account<'a>,
}

impl<'a> AmbientArgument<'a> for CommandSource<'a> {
    fn load_from(ctx: &'a dyn Command) -> Result<Self, CommandError> {
        Ok(ctx.source())
    }
}

impl<'a> AmbientArgument<'a> for UserSource<'a> {
    fn load_from(ctx: &'a dyn Command) -> Result<Self, CommandError> {
        match ctx.source() {
            CommandSource::User(user, conn) => Ok(Self {
                user,
                user_connection: conn,
            }),
            _ => numeric_error!(NotRegistered),
        }
    }
}

impl<'a> AmbientArgument<'a> for PreClientSource {
    fn load_from(ctx: &'a dyn Command) -> Result<Self, CommandError> {
        match ctx.source() {
            CommandSource::PreClient(pc) => Ok(Self(pc.clone())),
            _ => numeric_error!(AlreadyRegistered),
        }
    }
}

impl<'a> AmbientArgument<'a> for LoggedInUserSource<'a> {
    fn load_from(ctx: &'a dyn Command) -> Result<Self, CommandError> {
        match ctx.source() {
            CommandSource::User(user, user_connection) => {
                if let Some(account) = user.account()? {
                    Ok(Self {
                        user,
                        user_connection,
                        account,
                    })
                } else {
                    Err(CommandError::NotLoggedIn)
                }
            }
            CommandSource::PreClient(_) => numeric_error!(NotRegistered),
        }
    }
}

impl<'a> std::ops::Deref for UserSource<'a> {
    type Target = wrapper::User<'a>;

    fn deref(&self) -> &Self::Target {
        &self.user
    }
}

impl<'a> std::convert::AsRef<wrapper::User<'a>> for UserSource<'a> {
    fn as_ref(&self) -> &wrapper::User<'a> {
        &self.user
    }
}

impl<'a> From<UserSource<'a>> for wrapper::User<'a> {
    fn from(val: UserSource<'a>) -> Self {
        val.user
    }
}

impl std::ops::Deref for PreClientSource {
    type Target = PreClient;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl std::convert::AsRef<PreClient> for PreClientSource {
    fn as_ref(&self) -> &PreClient {
        self.0.as_ref()
    }
}

impl From<Arc<PreClient>> for PreClientSource {
    fn from(value: Arc<PreClient>) -> Self {
        Self(value)
    }
}

impl From<PreClientSource> for Arc<PreClient> {
    fn from(val: PreClientSource) -> Self {
        val.0
    }
}
