use crate::id::*;

#[derive(Debug)]
pub struct Message
{
    pub id: MessageId,
    pub source: UserId,
    pub target: ObjectId,
    pub text: String,
}

