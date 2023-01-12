use crate::prelude::*;
use serde::{Serialize,Deserialize};

#[derive(PartialEq,Debug,Clone,Serialize,Deserialize)]
pub struct ServicesData
{
    pub server_id: ServerId,
    pub sasl_mechanisms: Vec<String>,
}