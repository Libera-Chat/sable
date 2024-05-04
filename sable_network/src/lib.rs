#![feature(round_char_boundary)]
#![feature(hash_extract_if)]

pub mod prelude;

pub mod id;
pub mod modes;
pub mod validated;

pub mod history;

pub mod network;

pub mod policy;

pub mod rpc;

pub mod saveable;

pub mod sync;

pub mod node;

pub mod config;

pub mod audit;

pub mod types {
    mod matchers;
    pub use matchers::*;

    mod pattern;
    pub use pattern::*;
}

pub mod utils;

pub mod chert {
    pub use chert::compile::Engine;
    pub use chert::parse::nodes::boolean::NodeBoolean;
    pub use chert::*;
}

mod build_data {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
