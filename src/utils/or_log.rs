use log::error;
use std::fmt::{Debug,Display};

pub trait OrLog
{
    fn or_log(&self, context: impl Display);
}

impl<T, E: Debug> OrLog for Result<T,E>
{
    fn or_log(&self, context: impl Display)
    {
        if let Err(e) = &self
        {
            error!("Error: {:?} ({})", e, context);
        }
    }
}
