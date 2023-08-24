//! Services for sable networks
//!
//!

#![feature(return_position_impl_trait_in_trait)]
#![allow(incomplete_features)]

pub mod database;
mod model;

mod server;
pub use server::ServicesServer;
