use crate::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct ServicesData {
    pub server_id: ServerId,
    pub sasl_mechanisms: Vec<String>,
}
