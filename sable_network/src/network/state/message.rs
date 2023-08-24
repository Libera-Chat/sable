use crate::prelude::*;

use serde::{Deserialize, Serialize};

/// Message type - privmsg or notice
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MessageType {
    Privmsg,
    Notice,
}

/// A message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub source: UserId,
    pub target: ObjectId,
    pub ts: i64,
    pub message_type: MessageType,
    pub text: String,
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Privmsg => "PRIVMSG".fmt(f),
            Self::Notice => "NOTICE".fmt(f),
        }
    }
}
