pub mod config;
pub mod tracing_config;
mod strip_comments;

mod management
{
    mod command;
    pub use command::*;
    mod service;
    pub use service::*;
}

pub use sable_ircd::prelude::*;
pub use sable_network::prelude::*;
pub use structopt::StructOpt;

pub use tokio::{
    sync::mpsc::{
        channel,
        unbounded_channel,
    },
    sync::broadcast,
    time
};

pub use std::{
    fs::{
        File,
    },
    io::{
        BufReader,
    },
    path::{
        Path,
        PathBuf,
    },
    error::Error,
};
pub use serde::{
    Serialize,
    Deserialize
};

pub mod server;