use irc_server::Server;
use irc_network::{
    EpochId,
    ServerId,
    ServerName,
    EventIdGenerator,
};
use ircd_sync::*;
use structopt::StructOpt;

use tokio::{
    sync::mpsc::{
        channel
    },
    time
};

use std::{
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
use serde::Deserialize;

use log;
use simple_logger::SimpleLogger;

#[derive(Debug,StructOpt)]
#[structopt(rename_all = "kebab")]
struct Opts
{
    /// Network-wide config file location
    #[structopt(short,long)]
    network_conf: PathBuf,

    /// Server config file location
    #[structopt(short,long)]
    server_conf: PathBuf
}

#[derive(Debug,Deserialize)]
struct ServerConfig
{
    server_id: ServerId,
    server_name: ServerName,
    listen_addr: String,

    node_config: NodeConfig,
}

impl ServerConfig
{
    pub fn load_file<P: AsRef<Path>>(filename: P) -> Result<Self, ConfigError>
    {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let opts = Opts::from_args();

    let network_config = NetworkConfig::load_file(opts.network_conf)?;
    let server_config = ServerConfig::load_file(opts.server_conf)?;

    SimpleLogger::new().with_level(log::LevelFilter::Debug)
//                       .with_module_level("ircd_sync::network", log::LevelFilter::Trace)
                       .with_module_level("rustls", log::LevelFilter::Info)
                       .init().unwrap();

    let (server_send, server_recv) = channel(128);
    let (new_send, new_recv) = channel(128);
    let (shutdown_send, shutdown_recv) = channel(1);

    let id_gen = EventIdGenerator::new(server_config.server_id, EpochId::new(1), 0);
    let mut event_log = ReplicatedEventLog::new(id_gen, server_send, new_recv, network_config, server_config.node_config);

    let mut server = Server::new(server_config.server_id,
                                 server_config.server_name,
                                 server_recv,
                                 new_send);
    
    server.add_listener(server_config.listen_addr.parse().unwrap());

    ctrlc::set_handler(move || {
        shutdown_send.try_send(()).expect("Failed to send shutdown command");
    }).expect("Failed to set Ctrl+C handler");

    event_log.sync_to_network().await;

    tokio::spawn(event_log.sync_task());

    // Run the actual server
    server.run(shutdown_recv).await;
    // ...and once it shuts down, give the network sync some time to push the ServerQuit out
    time::sleep(std::time::Duration::new(1,0)).await;

    Ok(())
}