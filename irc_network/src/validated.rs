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

struct StringValidationError(String);
type StringValidationResult = Result<(), StringValidationError>;

fn check_allowed_chars(value: &str, allowed_chars: &[&str]) -> StringValidationResult
{
    for c in value.chars() {
        if ! allowed_chars.iter().any(|s| s.contains(c)) {
            return Err(StringValidationError(value.to_string()));
        }
    }
    Ok(())
}

const LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGIT: &str = "0123456789";

fn check_max_length(value: &str, max_len: usize) -> StringValidationResult
{
    if value.len() > max_len {
        Err(StringValidationError(value.to_string()))
    } else {
        Ok(())
    }
}

define_validated! {
    Nickname {
        check_max_length(value, 9)?;
        check_allowed_chars(value, &[LOWER, UPPER, DIGIT, "-_\\|[]{}^`"])?;
        if let Some(first) = value.chars().next() {
            if DIGIT.contains(first) || first == '-' {
                return Self::error(value);
            }
        } else {
            return Self::error(value);
        }
        Ok(())
    }

    Username {
        check_max_length(value, 10)?;
        Ok(())
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

    ServerName {
        check_allowed_chars(value, &[UPPER, LOWER, DIGIT, "_-."])?;
        if let Some(first) = value.chars().next() {
            if DIGIT.contains(first) || first == '-' {
                return Self::error(value);
            }
        } else {
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
