use sable_macros::object_ids;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Mismatched object ID type for event")]
pub struct WrongIdTypeError;

object_ids!(ConnectionObjectId {
    Listener: (i64,) sequential;
    Connection: (ListenerId,i64) sequential;
});
