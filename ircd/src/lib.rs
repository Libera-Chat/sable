//! This library exists to be used as a prelude package for command-line binaries,
//! just importing and re-exporting symbols that are likely to be needed.

pub use irc_network::{
    *,
    event::*,
};
pub use ircd_sync::*;
pub use structopt::StructOpt;

pub use tokio::{
    sync::mpsc::{
        channel
    },
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
};
pub use serde::Deserialize;

pub use log;
pub use simple_logger::SimpleLogger;