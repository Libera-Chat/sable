use sable_network::id::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct AccountAuth {
    pub account: AccountId,
    pub password_hash: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SaslSession {
    pub id: SaslSessionId,
    pub mechanism: String,
}
