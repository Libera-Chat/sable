use serde::{
    Serialize,
    de::DeserializeOwned
};

/// A type that can be converted to and from a serialisable form
pub trait Saveable
{
    /// The serialisable type into which this can be saved
    type Saved : Serialize + DeserializeOwned;

    /// Save the object into a serialisable form
    fn save(self) -> Self::Saved;

    /// Restore from a deserialised form
    fn restore(from: Self::Saved) -> Self;
}