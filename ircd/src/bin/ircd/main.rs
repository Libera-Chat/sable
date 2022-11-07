use ircd::*;
use ircd::config::*;
use ircd::server::*;
use client_listener::*;

use sable_network::rpc::ShutdownAction;

use std::{
    process::Command,
    env,
    os::unix::process::CommandExt,
};

use tracing_subscriber::util::SubscriberInitExt;

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

    /// Run in foreground without daemonising
    #[structopt(short,long)]
    foreground: bool
}

/// The async entry point for the application.
///
/// We can't use `[tokio::main]` because the tokio runtime can't survive daemonising,
/// so this is called after daemonising and manually initialising the runtime.
async fn sable_main(server_conf_path: &Path,
                    server_config: ServerConfig,
                    sync_conf_path: &Path,
                    sync_config: SyncConfig,
                    tls_data: TlsData,
                    upgrade_fd: Option<i32>,
                    bootstrap_network: Option<sable_network::network::config::NetworkConfig>,
                ) -> Result<(), Box<dyn std::error::Error>>
{
    let exe_path = std::env::current_exe()?;

    println!("uid={}", nix::unistd::getuid());

    ircd::tracing_config::build_subscriber(server_config.log.clone())?.init();

    let (client_send, client_recv) = unbounded_channel();

    let server = if let Some(upgrade_fd) = upgrade_fd
    {
        tracing::info!("Got upgrade FD {}", upgrade_fd);

        let state = upgrade::read_upgrade_state(upgrade_fd);

        let server = Server::restore_from(state,
                                          sync_config,
                                          server_config
                                    )?;

        server
    }
    else
    {
        let client_listeners = ListenerCollection::new(client_send)?;

        client_listeners.load_tls_certificates(tls_data.key.clone(), tls_data.cert_chain.clone())?;

        for listener in server_config.listeners.iter()
        {
            let conn_type = if listener.tls {ConnectionType::Tls} else {ConnectionType::Clear};
            client_listeners.add_listener(listener.address.parse().unwrap(), conn_type)?;
        }

        Server::new(server_config,
                    sync_config,
                    bootstrap_network,
                    client_listeners,
                    client_recv
            ).await
    };

    // Run the actual server
    let shutdown_action = server.run().await;

    // Now that we've closed down, deal with whatever the intended action was
    match shutdown_action
    {
        ShutdownAction::Shutdown =>
        {
            server.shutdown().await;

            Ok(())
        }
        ShutdownAction::Restart =>
        {
            server.shutdown().await;

            let err = Command::new(env::current_exe()?)
                        .args(env::args().skip(1).collect::<Vec<_>>())
                        .exec();

            panic!("Couldn't re-execute: {}", err);
        }
        ShutdownAction::Upgrade =>
        {
            let server_state = server.save().await;

            upgrade::exec_upgrade(&exe_path, server_conf_path, sync_conf_path, server_state);
        }
    }
}

/// Main entry point.
///
/// Because the tokio runtime can't survive forking, `main()` loads the application
/// configs (in order to report as many errors as possible before daemonising), daemonises,
/// initialises the tokio runtime, and begins the async entry point [`sable_main`].
pub fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let opts = Opts::from_args();

    let sync_config = SyncConfig::load_file(opts.network_conf.clone())?;
    let server_config = ServerConfig::load_file(opts.server_conf.clone())?;

    let tls_data = server_config.tls_config.load_from_disk()?;

    let bootstrap_conf = opts.bootstrap_network.map(load_network_config).transpose()?;

    if !server_config.log.dir.is_dir()
    {
        std::fs::create_dir_all(&server_config.log.dir).expect("failed to create log directory");
    }

    // Don't re-daemonise if we're upgrading; in that case if we're supposed to be daemonised then
    // we already are.
    if !opts.foreground && opts.upgrade_state_fd.is_none()
    {
        let mut daemon = daemonize::Daemonize::new()
                            .exit_action(|| println!("Running in background mode"))
                            .working_directory(std::env::current_dir()?);

        if let Some(stdout) = &server_config.log.stdout
        {
            daemon = daemon.stdout(File::create(&server_config.log.prefix_file(stdout)).unwrap());
        }
        if let Some(stderr) = &server_config.log.stderr
        {
            daemon = daemon.stderr(File::create(&server_config.log.prefix_file(stderr)).unwrap());
        }
        if let Some(pidfile) = &server_config.log.pidfile
        {
            daemon = daemon.pid_file(server_config.log.prefix_file(pidfile));
        }

        daemon.start().expect("Failed to fork to background");
    }

    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(sable_main(&opts.server_conf,
                                server_config,
                                &opts.network_conf,
                                sync_config,
                                tls_data,
                                opts.upgrade_state_fd,
                                bootstrap_conf,
                            ))

}