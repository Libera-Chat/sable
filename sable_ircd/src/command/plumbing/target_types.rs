use super::*;
use crate::numeric_error;
use std::str::FromStr;

pub enum TargetParameter<'a>
{
    User(wrapper::User<'a>),
    Channel(wrapper::Channel<'a>)
}

impl TargetParameter<'_>
{
    pub fn user(&self) -> Option<&wrapper::User>
    {
        match self {
            Self::User(u) => Some(&u),
            Self::Channel(_) => None
        }
    }

    pub fn channel(&self) -> Option<&wrapper::Channel>
    {
        match self {
            Self::User(_) => None,
            Self::Channel(c) => Some(&c)
        }
    }

    pub fn object_id(&self) -> ObjectId
    {
        match self {
            Self::User(u) => u.id().into(),
            Self::Channel(c) => c.id().into()
        }
    }
}

impl<'a> ArgumentType<'a> for TargetParameter<'a>
{
    type Category = PositionalArgumentType<Self>;
    fn parse_str(ctx: &'a impl CommandContext, value: &'a str) -> Result<Self, CommandError>
    {
        if let Ok(chname) = ChannelName::from_str(value)
        {
            let net = ctx.network();
            Ok(Self::Channel(net.channel_by_name(&chname)?))
        }
        else if let Ok(nick) = Nickname::from_str(value)
        {
            Ok(Self::User(ctx.network().user_by_nick(&nick)?))
        }
        else
        {
            numeric_error!(NoSuchTarget, value)
        }
    }
}