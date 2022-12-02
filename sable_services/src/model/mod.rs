use sable_network::{
    id::*,
};
use serde::{Serialize,Deserialize};

#[derive(Clone,Serialize,Deserialize)]
pub struct AccountAuth
{
    pub account: AccountId,
    pub password_hash: String,
}