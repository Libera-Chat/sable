//! Defines validated string types for various names and identifiers

use ircd_macros::define_validated;
use thiserror::Error;
use arrayvec::ArrayString;
use std::convert::{TryFrom,Into};

/// Base trait for validated string types.
pub trait Validated : TryFrom<Self::Underlying> + Into<Self::Underlying> + Sized
{
    type Underlying;
    type Error;
    type Result;

    /// Check whether the provided value is valid according to this type's
    /// rules.
    fn validate(value: &Self::Underlying) -> Result<(), <Self as Validated>::Error>;

    /// Attempt to create a new instance using the given value. Returns `Ok(_)`
    /// if the value passes validation, and `Err(_)` if not.
    fn new(value: Self::Underlying) -> <Self as Validated>::Result;

    /// Access the raw stored value
    fn value(&self) -> &Self::Underlying;

    /// Attempt to convert from [&str] to the underlying type, then validate
    /// the resulting value, and construct a new `Self` if it succeeds.
    fn from_str(value: &str) -> Self::Result;

    /// Attempt to convert from anything that can be converted to a string.
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
    Nickname(ArrayString<15> casefolded) {
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

    ChannelName(ArrayString<64> casefolded) {
        if value.starts_with('#') || value.starts_with('&')
        {
            Ok(())
        }
        else
        {
            Self::error(value)
        }
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

    ChannelKey(ArrayString<64>) {
        for c in value.chars()
        {
            // less than 0x20 (' ') is control chars, greater than 0x7E (~) is outside of ascii
            // colon, comma and space break protocol parsing
            if c <= ' ' || c > '~' || c == ':' || c == ','
            {
                return Self::error(value);
            }
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
    /// Coerce the provided value into a valid `Username`, by truncating to the
    /// permitted length and removing any invalid characters.
    pub fn new_coerce(s: &str) -> Self
    {
        let mut s = s.to_string();
        s.retain(|c| c != '[');
        s.truncate(10);
        // expect() is safe here; we've already truncated to the max length
        Self(ArrayString::try_from(s.as_str()).expect("Failed to convert string"))
    }
}

impl ChannelKey
{
    pub fn new_coerce(s: &str) -> Self
    {
        let mut s = s.to_string();
        s.retain(|c| c > ' ' && c <= '~' && c != ':' && c != ',');
        let mut val = <Self as Validated>::Underlying::new();
        s.truncate(val.capacity());
        val.push_str(&s);
        Self(val)
    }
}
