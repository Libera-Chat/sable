use ircd_macros::define_validated;
use thiserror::Error;
use std::convert::{TryFrom,Into};

pub trait Validated : TryFrom<Self::Underlying> + Into<Self::Underlying> where Self: Sized
{
    type Underlying;
    type Error;
    type Result;

    fn validate(value: &Self::Underlying) -> Result<(), <Self as Validated>::Error>;
    fn new(value: Self::Underlying) -> <Self as Validated>::Result;
    fn value(&self) -> &Self::Underlying;
}

define_validated! {
    Nickname {
        if value.len() > 9 {
            Self::error(value)
        } else {
            Ok(())
        }
    }

    Username {
        if value.len() > 10 {
            Self::error(value)
        } else {
            Ok(())
        }
    }

    Hostname {
        Ok(())
    }

    ChannelName {
        if ! value.starts_with("#") {
            return Self::error(value);
        }
        Ok(())
    }
}

impl Username
{
    pub fn new_coerce(s: &str) -> Self
    {
        let mut s = s.to_string();
        s.retain(|c| c != '[');
        s.truncate(10);
        Self(s)
    }
}
