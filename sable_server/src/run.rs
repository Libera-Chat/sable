use sable_network::{rpc::ShutdownAction, sync::SyncConfig};
use crate::{Server, ServerConfig, ServerType, config::load_network_config};

use std::{
    process::Command,
    env,
    os::unix::{
        process::CommandExt,
        prelude::RawFd,
        io::{IntoRawFd,FromRawFd},
    },
    path::Path,
    fs::File,
    io::Seek,
};

use tracing_subscriber::util::SubscriberInitExt;

use memfd::*;

use crate::ServerState;

pub fn read_upgrade_state<ST: ServerType>(fd: RawFd) -> ServerState<ST>
{
    let memfd = unsafe { Memfd::from_raw_fd(fd) };
    let file = memfd.as_file();

    serde_json::from_reader(file).expect("Failed to unpack upgrade state")
}

fn prepare_upgrade<ST: ServerType>(state: ServerState<ST>) -> RawFd
{
    let memfd = MemfdOptions::default().close_on_exec(false).create("upgrade_state").expect("Failed to create upgrade memfd");
    let mut file = memfd.as_file();

    serde_json::to_writer(file, &state).expect("Failed to serialise server state");
    file.rewind().expect("Failed to rewind memfd");
    memfd.into_raw_fd()
}

pub(super) fn exec_upgrade<ST>(exe: impl AsRef<Path>,
                               server_conf: impl AsRef<Path>,
                               network_conf: impl AsRef<Path>,
                               state: ServerState<ST>
                    ) -> !
    where ST: ServerType
{
    let fd = prepare_upgrade(state);
    let args = ["--server-conf",
                server_conf.as_ref().to_str().unwrap(),
                "--network-conf",
                network_conf.as_ref().to_str().unwrap(),
                "--upgrade-state-fd",
                &fd.to_string()];

    tracing::debug!("Executing upgrade: {:?} {:?}", exe.as_ref(), args);

    let err = Command::new(exe.as_ref())
                      .args(args)
                      .exec();

    panic!("exec() failed on upgrade: {}", err);
}

/// The async entry point for the application.
///
/// We can't use `[tokio::main]` because the tokio runtime can't survive daemonising,
/// so this is called after daemonising and manually initialising the runtime.
async fn sable_main<ST>(server_conf_path: impl AsRef<Path>,
                        server_config: ServerConfig<ST>,
                        sync_conf_path: impl AsRef<Path>,
                        sync_config: SyncConfig,
                        tls_data: sable_network::config::TlsData,
                        upgrade_fd: Option<i32>,
                        bootstrap_network: Option<sable_network::network::config::NetworkConfig>,
                    ) -> Result<(), Box<dyn std::error::Error>>
    where ST: ServerType
{
    let exe_path = std::env::current_exe()?;

    println!("uid={}", nix::unistd::getuid());

    crate::tracing_config::build_subscriber(server_config.log.clone())?.init();

    let server = if let Some(upgrade_fd) = upgrade_fd
    {
        tracing::info!("Got upgrade FD {}", upgrade_fd);

        let state = read_upgrade_state(upgrade_fd);

        let server = Server::restore_from(state,
                                          sync_config,
                                          server_config
                                    )?;

        server
    }
    else
    {
        Server::new(server_config,
                    tls_data,
                    sync_config,
                    bootstrap_network,
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

            exec_upgrade(&exe_path, server_conf_path, sync_conf_path, server_state);
        }
    }
}

/// Main entry point.
///
/// Because the tokio runtime can't survive forking, `main()` loads the application
/// configs (in order to report as many errors as possible before daemonising), daemonises,
/// initialises the tokio runtime, and begins the async entry point [`sable_main`].
pub fn run_server<ST>(server_config_path: impl AsRef<Path>,
                      sync_config_path: impl AsRef<Path>,
                      foreground: bool,
                      upgrade_fd: Option<RawFd>,
                      bootstrap_config: Option<impl AsRef<Path>>,
            ) -> Result<(), Box<dyn std::error::Error>>
    where ST: ServerType
{
    let sync_config = SyncConfig::load_file(&sync_config_path)?;
    let server_config = ServerConfig::<ST>::load_file(&server_config_path)?;

    let tls_data = server_config.tls_config.load_from_disk()?;

    let bootstrap_conf = bootstrap_config.map(load_network_config).transpose()?;

    if !server_config.log.dir.is_dir()
    {
        std::fs::create_dir_all(&server_config.log.dir).expect("failed to create log directory");
    }

    // Don't re-daemonise if we're upgrading; in that case if we're supposed to be daemonised then
    // we already are.
    if !foreground && upgrade_fd.is_none()
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

    runtime.block_on(sable_main(&server_config_path,
                                server_config,
                                &sync_config_path,
                                sync_config,
                                tls_data,
                                upgrade_fd,
                                bootstrap_conf,
                            ))

}