use ircd_macros::define_validated;
use thiserror::Error;
use arrayvec::ArrayString;
use std::convert::{TryFrom,Into};

pub trait Validated : TryFrom<Self::Underlying> + Into<Self::Underlying> + Sized
{
    type Underlying;
    type Error;
    type Result;

    fn validate(value: &Self::Underlying) -> Result<(), <Self as Validated>::Error>;
    fn new(value: Self::Underlying) -> <Self as Validated>::Result;
    fn value(&self) -> &Self::Underlying;
    fn from_str(value: &str) -> Self::Result;
    fn convert(arg: impl std::string::ToString) -> Self::Result;
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

define_validated! {
    Nickname(ArrayString<15>) {
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

    Username(ArrayString<10>) {
        Ok(())
    }

    Hostname(ArrayString<64>) {
        Ok(())
    }

    ChannelName(ArrayString<64>) {
        if ! value.starts_with("#") {
            return Self::error(value);
        }
        Ok(())
    }

    ServerName(ArrayString<64>) {
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

impl Nickname
{
    /// Create a new Nickname, bypassing normal validation. This is only for internal use, and only when created
    /// nicknames for collided users
    pub(crate) fn new_for_collision(value: <Self as Validated>::Underlying) -> <Self as Validated>::Result
    {
        Ok(Self(value))
    }
}

impl Username
{
    pub fn new_coerce(s: &str) -> Self
    {
        let mut s = s.to_string();
        s.retain(|c| c != '[');
        s.truncate(10);
        // expect() is safe here; we've already truncated to the max length
        Self(ArrayString::try_from(s.as_str()).expect("Failed to convert string"))
    }
}
