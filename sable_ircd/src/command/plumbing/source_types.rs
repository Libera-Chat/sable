use crate::{client::PreClient, numeric_error};

use super::*;

pub struct UserSource<'a>(pub wrapper::User<'a>);
pub struct PreClientSource(pub Arc<PreClient>);

impl<'a> ArgumentType<'a> for CommandSource<'a>
{
    type Category = AmbientArgumentType<Self>;
    fn parse_ambient(ctx: &'a impl CommandContext) -> Result<Self, CommandError>
    {
        Ok(ctx.source())
    }
}

impl<'a> ArgumentType<'a> for UserSource<'a>
{
    type Category = AmbientArgumentType<Self>;
    fn parse_ambient(ctx: &'a impl CommandContext) -> Result<Self, CommandError>
    {
        match ctx.source()
        {
            CommandSource::User(user) => Ok(Self(user)),
            _ => numeric_error!(NotRegistered)
        }
    }
}

impl<'a> ArgumentType<'a> for PreClientSource
{
    type Category = AmbientArgumentType<Self>;
    fn parse_ambient(ctx: &'a impl CommandContext) -> Result<Self, CommandError>
    {
        match ctx.source()
        {
            CommandSource::PreClient(pc) => Ok(Self(pc.clone())),
            _ => numeric_error!(AlreadyRegistered)
        }
    }
}

impl<'a> std::ops::Deref for UserSource<'a>
{
    type Target: = wrapper::User<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> std::convert::AsRef<wrapper::User<'a>> for UserSource<'a>
{
    fn as_ref(&self) -> &wrapper::User<'a> {
        &self.0
    }
}

impl<'a> From<wrapper::User<'a>> for UserSource<'a>
{
    fn from(value: wrapper::User<'a>) -> Self {
        Self(value)
    }
}

impl<'a> Into<wrapper::User<'a>> for UserSource<'a>
{
    fn into(self) -> wrapper::User<'a> {
        self.0
    }
}

impl std::ops::Deref for PreClientSource
{
    type Target: = PreClient;

    fn deref(&self) -> &Self::Target {
        &self.0.deref()
    }
}

impl std::convert::AsRef<PreClient> for PreClientSource
{
    fn as_ref(&self) -> &PreClient {
        self.0.as_ref()
    }
}

impl From<Arc<PreClient>> for PreClientSource
{
    fn from(value: Arc<PreClient>) -> Self {
        Self(value)
    }
}

impl Into<Arc<PreClient>> for PreClientSource
{
    fn into(self) -> Arc<PreClient> {
        self.0
    }
}