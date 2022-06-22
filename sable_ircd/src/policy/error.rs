use super::*;

/// Possible return values from a failed permission check
#[derive(Debug)]
pub enum PermissionError
{
    /// A numeric response, which will be sent to the source client
    Numeric(Box<dyn Numeric>),
    /// The permission check failed, but there is no standard numeric to say so.
    /// This should be used if the user has already been informed of the reason;
    /// no further notification will be sent.
    CustomError,
    /// Some internal error occurred
    InternalError(Box<dyn std::error::Error>),
}

impl<T: Numeric + 'static> From<T> for PermissionError
{
    fn from(e: T) -> Self {
        Self::Numeric(Box::new(e))
    }
}

impl From<LookupError> for PermissionError
{
    fn from(e: LookupError) -> Self { Self::InternalError(Box::new(e)) }
}