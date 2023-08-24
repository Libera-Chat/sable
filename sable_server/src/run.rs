use crate::{config::load_network_config, Server, ServerConfig, ServerType};
use sable_network::{rpc::ShutdownAction, sync::SyncConfig};

use std::{
    env,
    fs::File,
    io::Seek,
    os::unix::{
        io::{FromRawFd, IntoRawFd},
        prelude::RawFd,
        process::CommandExt,
    },
    path::Path,
    process::Command,
};

use tracing_subscriber::util::SubscriberInitExt;

use memfd::*;

use crate::ServerState;

fn read_upgrade_state<ST: ServerType>(fd: RawFd) -> ServerState<ST> {
    let memfd = unsafe { Memfd::from_raw_fd(fd) };
    let file = memfd.as_file();

    serde_json::from_reader(file).expect("Failed to unpack upgrade state")
}

fn prepare_upgrade<ST: ServerType>(state: ServerState<ST>) -> RawFd {
    let memfd = MemfdOptions::default()
        .close_on_exec(false)
        .create("upgrade_state")
        .expect("Failed to create upgrade memfd");
    let mut file = memfd.as_file();

    serde_json::to_writer(file, &state).expect("Failed to serialise server state");
    file.rewind().expect("Failed to rewind memfd");
    memfd.into_raw_fd()
}

fn exec_upgrade<ST>(
    exe: impl AsRef<Path>,
    server_conf: impl AsRef<Path>,
    network_conf: impl AsRef<Path>,
    state: ServerState<ST>,
) -> !
where
    ST: ServerType,
{
    let fd = prepare_upgrade(state);
    let args = [
        "--server-conf",
        server_conf.as_ref().to_str().unwrap(),
        "--network-conf",
        network_conf.as_ref().to_str().unwrap(),
        "--upgrade-state-fd",
        &fd.to_string(),
    ];

    tracing::debug!("Executing upgrade: {:?} {:?}", exe.as_ref(), args);

    let err = Command::new(exe.as_ref()).args(args).exec();

    panic!("exec() failed on upgrade: {}", err);
}

// The async entry point for the application. Because `run_server` can fork into the background
// depending on options, it needs to initialise the tokio runtime after doing so
async fn do_run_server<ST>(
    server_conf_path: impl AsRef<Path>,
    server_config: ServerConfig<ST>,
    sync_conf_path: impl AsRef<Path>,
    sync_config: SyncConfig,
    tls_data: sable_network::config::TlsData,
    upgrade_fd: Option<i32>,
    bootstrap_network: Option<sable_network::network::config::NetworkConfig>,
) -> Result<(), Box<dyn std::error::Error>>
where
    ST: ServerType,
{
    let exe_path = std::env::current_exe()?;

    println!("uid={}", nix::unistd::getuid());

    crate::tracing_config::build_subscriber(server_config.log.clone())?.init();

    let server = if let Some(upgrade_fd) = upgrade_fd {
        tracing::info!("Got upgrade FD {}", upgrade_fd);

        let state = read_upgrade_state(upgrade_fd);

        let server = Server::restore_from(state, sync_config, server_config)?;

        server
    } else {
        Server::new(server_config, tls_data, sync_config, bootstrap_network).await
    };

    // Run the actual server
    let shutdown_action = server.run().await;

    // Now that we've closed down, deal with whatever the intended action was
    match shutdown_action {
        ShutdownAction::Shutdown => {
            server.shutdown().await;

            Ok(())
        }
        ShutdownAction::Restart => {
            server.shutdown().await;

            let err = Command::new(env::current_exe()?)
                .args(env::args().skip(1).collect::<Vec<_>>())
                .exec();

            panic!("Couldn't re-execute: {}", err);
        }
        ShutdownAction::Upgrade => {
            let server_state = server.save().await;

            exec_upgrade(&exe_path, server_conf_path, sync_conf_path, server_state);
        }
    }
}

/// Run a network server.
///
/// This function will load a `ServerConfig<ST>` from the provided `server_config_path`, a `SyncConfig`
/// from `sync_config_path`, and optionally a bootstrapping network config from `bootstrap_config`.
///
/// If `bootstrap_config` is `Some`, then an empty network state will be created, with the network
/// configuration from the provided file path. If it is `None`, then the new server will sync to an existing
/// network according to the configuration at the provided `sync_config_path`.
///
/// If `upgrade_fd` is `Some`, then a saved server state will be read from it and used to resume
/// processing after an in-place upgrade. In this case, `bootstrap_config` will not be used, but must still
/// be readable if it is supplied.
///
/// If `foreground` is false and `upgrade_fd` is None, then this function will daemonise before initialising
/// the tokio runtime. Standard output and error will be redirected to the files defined in the server config's
/// logging section, if any. If `upgrade_fd` is Some, then it is assumed that the server's previous execution
/// did any daemonisation that may have been required.
///
/// Note that this function will create a new tokio runtime. It should not be called if one is already active.
///
/// The `ST` generic parameter should be a type which implements [`ServerType`] and will be constructed to handle
/// application-specific functionality. The `server` field of the provided server config must contain appropriate
/// data to read an instance of `ST::Config`.
///
pub fn run_server<ST>(
    server_config_path: impl AsRef<Path>,
    sync_config_path: impl AsRef<Path>,
    foreground: bool,
    upgrade_fd: Option<RawFd>,
    bootstrap_config: Option<impl AsRef<Path>>,
) -> Result<(), Box<dyn std::error::Error>>
where
    ST: ServerType,
{
    // NB: Because the tokio runtime can't survive forking, `run_server()` loads the application
    // configs (in order to report as many errors as possible before daemonising), daemonises,
    // initialises the tokio runtime, and begins the async entry point [`sable_main`].

    let sync_config = SyncConfig::load_file(&sync_config_path)?;
    let server_config = ServerConfig::<ST>::load_file(&server_config_path)?;

    let tls_data = server_config.tls_config.load_from_disk()?;

    let bootstrap_conf = bootstrap_config.map(load_network_config).transpose()?;

    if !server_config.log.dir.is_dir() {
        std::fs::create_dir_all(&server_config.log.dir).expect("failed to create log directory");
    }

    // Don't re-daemonise if we're upgrading; in that case if we're supposed to be daemonised then
    // we already are.
    if !foreground && upgrade_fd.is_none() {
        let mut daemon = daemonize::Daemonize::new()
            .exit_action(|| println!("Running in background mode"))
            .working_directory(std::env::current_dir()?);

        if let Some(stdout) = &server_config.log.stdout {
            daemon = daemon.stdout(File::create(&server_config.log.prefix_file(stdout)).unwrap());
        }
        if let Some(stderr) = &server_config.log.stderr {
            daemon = daemon.stderr(File::create(&server_config.log.prefix_file(stderr)).unwrap());
        }
        if let Some(pidfile) = &server_config.log.pidfile {
            daemon = daemon.pid_file(server_config.log.prefix_file(pidfile));
        }

        daemon.start().expect("Failed to fork to background");
    }

    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(do_run_server(
        &server_config_path,
        server_config,
        &sync_config_path,
        sync_config,
        tls_data,
        upgrade_fd,
        bootstrap_conf,
    ))
}
