use crate::id::*;
use serde::{
    Serialize,
    Deserialize
};

/// A message
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Message
{
    pub id: MessageId,
    pub source: UserId,
    pub target: ObjectId,
    pub text: String,
}

