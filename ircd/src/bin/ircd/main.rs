use ircd::*;
use ircd::config::*;
use client_listener::*;

use sable_network::rpc::ShutdownAction;

use std::{
    process::Command,
    env,
    os::unix::process::CommandExt,
    sync::Arc,
};

use tokio::{
    sync::broadcast,
    select
};
use tracing::Instrument;
use tracing_subscriber::util::SubscriberInitExt;

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

    ircd::tracing_config::build_subscriber(server_config.log)?.init();

    let (client_send, client_recv) = unbounded_channel();
    let (server_send, server_recv) = unbounded_channel();

    // There are two shutdown channels, one for the Server task and one for everything else.
    // The Server needs to shut down first, because it'll panic if any of the others disappears
    // from under it.
    let (shutdown_send, _shutdown_recv) = broadcast::channel(1);
    let (server_shutdown_send, server_shutdown_recv) = oneshot::channel();

    let (event_log, client_listeners, mut server) = if let Some(upgrade_fd) = upgrade_fd {
        tracing::info!("Got upgrade FD {}", upgrade_fd);

        let state = upgrade::read_upgrade_state(upgrade_fd);

        let event_log = Arc::new(ReplicatedEventLog::restore(state.sync_state, server_send, sync_config, server_config.node_config));

        let client_listeners = ListenerCollection::resume(state.listener_state, client_send)?;

        let server = ClientServer::restore_from(state.server_state,
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

        let network = if let Some(net_conf) = bootstrap_network {
            Network::new(net_conf)
        } else {
            *event_log.sync_to_network().await
        };

        let server = ClientServer::new(server_config.server_id,
                                 epoch,
                                 server_config.server_name,
                                 network,
                                 Arc::clone(&event_log),
                                 server_recv,
                                 client_recv,
                            );

        client_listeners.load_tls_certificates(tls_data.key.clone(), tls_data.cert_chain.clone())?;

        for listener in server_config.listeners
        {
            let conn_type = if listener.tls {ConnectionType::Tls} else {ConnectionType::Clear};
            client_listeners.add_listener(listener.address.parse().unwrap(), conn_type)?;
        }

        (event_log, client_listeners, server)
    };

    let (management_send, management_recv) = channel(128);
    let management_config = server_config.management.clone();

    let management_shutdown = shutdown_send.subscribe();
    let (mgmt_task_shutdown, mut mgmt_task_shutdown_recv) = oneshot::channel();

    let management_task = tokio::spawn(async move {
        let mut server = management::ManagementServer::start(management_config, tls_data, management_shutdown);

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

            upgrade::exec_upgrade(&exe_path, server_conf_path, sync_conf_path, upgrade::ApplicationState {
                server_state,
                listener_state,
                sync_state
            });
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

    let bootstrap_conf = opts.bootstrap_network.map(|path| load_network_config(path)).transpose()?;

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