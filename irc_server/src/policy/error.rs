use super::*;

#[derive(Debug)]
pub enum PermissionError
{
    Numeric(Box<dyn Numeric>),
    CustomError,
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