use ircd::*;
use irc_server::{
    Server,
};
use client_listener::*;
use rpc_protocols::ShutdownAction;

use std::{
    process::Command,
    env,
    os::unix::process::CommandExt,
    net::SocketAddr,
    sync::Arc,
};

use tokio::{
    sync::broadcast,
    select
};
use tracing::Instrument;

mod management
{
    mod command;
    pub use command::*;
    mod service;
    pub use service::*;
}
mod upgrade;

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

    /// FD from which to read upgrade data
    #[structopt(long)]
    upgrade_state_fd: Option<i32>,

    /// Start a new network; without this no clients will be accepted until the
    /// server has synced to an existing net
    #[structopt(long)]
    bootstrap_network: Option<PathBuf>,
}

#[derive(Debug,Deserialize)]
struct TlsConfig
{
    key_file: PathBuf,
    cert_file: PathBuf,
}

#[derive(Debug,Deserialize)]
struct ListenerConfig
{
    address: String,
    #[serde(default)]
    tls: bool,
}

#[derive(Debug,Deserialize)]
struct ServerConfig
{
    server_id: ServerId,
    server_name: ServerName,

    management_address: String,
    console_address: String,

    listeners: Vec<ListenerConfig>,

    tls_config: Option<TlsConfig>,
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

fn load_tls_server_config(conf: &TlsConfig) -> Result<client_listener::TlsSettings, Box<dyn Error>>
{
    let cert_file = File::open(&conf.cert_file)?;
    let mut cert_reader = BufReader::new(cert_file);
    let cert_chain = rustls_pemfile::certs(&mut cert_reader)?;

    let key_file = File::open(&conf.key_file)?;
    let mut key_reader = BufReader::new(key_file);

    let server_key = rustls_pemfile::read_one(&mut key_reader)?;

    use rustls_pemfile::Item;

    let server_key = match server_key {
        Some(Item::RSAKey(key)) | Some(Item::PKCS8Key(key)) => Ok(key),
        Some(Item::X509Certificate(_)) | None => Err(ConfigError::FormatError("No private key in file".to_string()))
    }?;

    Ok(client_listener::TlsSettings { key: server_key, cert_chain })
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
    //tracing_subscriber::fmt::init();

    let opts = Opts::from_args();
    let exe_path = std::env::current_exe()?;

    let sync_config = SyncConfig::load_file(opts.network_conf.clone())?;
    let server_config = ServerConfig::load_file(opts.server_conf.clone())?;

    console_subscriber::ConsoleLayer::builder()
        .server_addr(server_config.console_address.parse::<SocketAddr>()?)
        .init();

    let (client_send, client_recv) = channel(128);
    let (server_send, server_recv) = channel(128);

    // There are two shutdown channels, one for the Server task and one for everything else.
    // The Server needs to shut down first, because it'll panic if any of the others disappears
    // from under it.
    let (shutdown_send, _shutdown_recv) = broadcast::channel(1);
    let (server_shutdown_send, server_shutdown_recv) = oneshot::channel();

    let (event_log, client_listeners, mut server) = if let Some(upgrade_fd) = opts.upgrade_state_fd {
        tracing::info!("Got upgrade FD {}", upgrade_fd);

        let state = upgrade::read_upgrade_state(upgrade_fd);

        let event_log = Arc::new(ReplicatedEventLog::restore(state.sync_state, server_send, sync_config, server_config.node_config));

        let client_listeners = ListenerCollection::resume(state.listener_state, client_send)?;

        let server = Server::restore_from(state.server_state,
                                          Arc::clone(&event_log),
                                          &client_listeners,
                                          client_recv,
                                          server_recv
                                    )?;

        (event_log, client_listeners, server)
    }
    else
    {
        let epoch = EpochId::new(chrono::Utc::now().timestamp());
        let event_log = Arc::new(ReplicatedEventLog::new(
                                    server_config.server_id,
                                    epoch,
                                    server_send,
                                    sync_config,
                                    server_config.node_config
                                ));

        let client_listeners = ListenerCollection::new(client_send)?;

        let network = if let Some(net_conf_path) = &opts.bootstrap_network {
            Network::new(load_network_config(net_conf_path)?)
        } else {
            *event_log.sync_to_network().await
        };

        let server = Server::new(server_config.server_id,
                                 epoch,
                                 server_config.server_name,
                                 network,
                                 Arc::clone(&event_log),
                                 client_recv,
                                 server_recv,
                            );

        if let Some(conf) = server_config.tls_config {
            let tls_conf = load_tls_server_config(&conf)?;
            client_listeners.load_tls_certificates(tls_conf)?;
        }

        for listener in server_config.listeners
        {
            let conn_type = if listener.tls {ConnectionType::Tls} else {ConnectionType::Clear};
            client_listeners.add_listener(listener.address.parse().unwrap(), conn_type)?;
        }

        (event_log, client_listeners, server)
    };

    let (management_send, management_recv) = channel(128);
    let management_address = server_config.management_address.clone();

    let management_shutdown = shutdown_send.subscribe();
    let (mgmt_task_shutdown, mut mgmt_task_shutdown_recv) = oneshot::channel();

    let management_task = tokio::spawn(async move {
        let mut server = management::ManagementServer::start(management_address.parse().unwrap(), management_shutdown);

        let mut server_shutdown_send = Some(server_shutdown_send);
        loop
        {
            select!(
                res = server.recv() =>
                {
                    if let Some(cmd) = res
                    {
                        match cmd
                        {
                            management::ManagementCommand::ServerCommand(scmd) =>
                            {
                                management_send.send(scmd).await.ok();
                            }
                            management::ManagementCommand::Shutdown(action) =>
                            {
                                if let Some(sender) = server_shutdown_send.take()
                                {
                                    sender.send(action.clone()).ok();
                                }
                            }
                        }
                    }
                    else
                    {
                        break;
                    }
                },
                _ = &mut mgmt_task_shutdown_recv =>
                {
                    return server;
                }
            );
        }
        tracing::error!("Lost management server; shutting down");
        if let Some(sender) = server_shutdown_send.take() {
            sender.send(ShutdownAction::Shutdown).expect("Error signalling server shutdown");
        }
        server
    }.instrument(tracing::info_span!("management event pump")));

    let sync_task = event_log.start_sync(shutdown_send.subscribe());

    // Run the actual server - we don't use spawn() here because Server isn't Send/Sync
    let shutdown_action = server.run(management_recv, server_shutdown_recv).await;

    // ...and once it finishes, shut down the other tasks
    mgmt_task_shutdown.send(()).expect("Couldn't signal shutdown");
    let management_server = management_task.await?;

    shutdown_send.send(shutdown_action.clone())?;
    let (_,_) = tokio::join!(
        sync_task,
        management_server.wait()
    );

    // Now that we've closed down, deal with whatever the intended action was
    match shutdown_action
    {
        ShutdownAction::Shutdown =>
        {
            client_listeners.shutdown().await;
            Ok(())
        }
        ShutdownAction::Restart =>
        {
            client_listeners.shutdown().await;

            let err = Command::new(env::current_exe()?)
                        .args(env::args().skip(1).collect::<Vec<_>>())
                        .exec();

            panic!("Couldn't re-execute: {}", err);
        }
        ShutdownAction::Upgrade =>
        {
            let server_state = server.save_state().await?;
            let listener_state = client_listeners.save().await?;
            // Now that the Server has been consumed to turn it into a saved state,
            // its reference to the event log is gone, and we can unwrap the Arc
            let sync_state = Arc::try_unwrap(event_log)
                                 .unwrap_or_else(|_| panic!("Failed to unwrap event log"))
                                 .save_state()
                                 .expect("Failed to save event log state");

            upgrade::exec_upgrade(&exe_path, opts, upgrade::ApplicationState {
                server_state,
                listener_state,
                sync_state
            });
        }
    }
}