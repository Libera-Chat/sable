use ircd::*;
use chrono::prelude::*;

#[derive(Debug,StructOpt)]
#[structopt(rename_all = "kebab")]
struct Opts
{
    /// Network-wide config file location
    #[structopt(short,long)]
    network_conf: PathBuf,

    /// Server config file location
    #[structopt(short,long)]
    server_conf: PathBuf,

    /// Config file to load into network
    config_to_load: PathBuf,
}

#[derive(Debug,Deserialize)]
struct ServerConfig
{
    server_id: ServerId,
    server_name: ServerName,

    node_config: ircd_sync::NodeConfig,
}

impl ServerConfig
{
    pub fn load_file<P: AsRef<Path>>(filename: P) -> Result<Self, ircd_sync::ConfigError>
    {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }
}

fn load_network_config(filename: impl AsRef<Path>) -> Result<irc_network::config::NetworkConfig, ircd_sync::ConfigError>
{
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let opts = Opts::from_args();

    let network_config = ircd_sync::NetworkConfig::load_file(opts.network_conf)?;
    let server_config = ServerConfig::load_file(opts.server_conf)?;

    let config_to_load = load_network_config(opts.config_to_load)?;

    tracing_subscriber::fmt::init();

    let (msg_send, _msg_recv) = channel(128);

    let net = ircd_sync::Network::new(network_config, server_config.node_config, msg_send);

    let now = Utc::now().timestamp();
    let epoch = EpochId::new(now);

    let event = Event {
        id: EventId::new(server_config.server_id, epoch, 1),
        target: ConfigId::new(1).into(),
        timestamp: now,
        clock: EventClock::new(),
        details: details::LoadConfig {
            config: config_to_load
        }.into()
    };

    net.propagate(&ircd_sync::Message {
        source_server: (server_config.server_id, epoch),
        content: ircd_sync::MessageDetail::NewEvent(event)
    }).await;

    // Because we discarded the receive half of the channel above, the normal process of
    // exchanging messages back and forth will fail, so exiting immediately here will cause
    // read errors on the target server(s). Give them time to process it.
    time::sleep(std::time::Duration::new(1,0)).await;

    Ok(())
}