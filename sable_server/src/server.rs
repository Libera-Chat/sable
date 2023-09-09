use crate::{config::*, *};

use anyhow::Context;
use parking_lot::Mutex;
use sable_network::{
    config::*,
    network::config::NetworkConfig,
    node::NetworkNodeState,
    policy::StandardPolicyService,
    prelude::*,
    rpc::{RemoteServerRequest, ShutdownAction},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::sync::{
    broadcast,
    mpsc::{unbounded_channel, UnboundedReceiver},
    oneshot,
};

use std::{fs::File, io::Read, path::Path, sync::Arc};

/// Configuration for a network server
#[derive(Debug, Deserialize)]
pub struct ServerConfig<ST>
where
    ST: ServerType,
{
    pub server_id: ServerId,
    pub server_name: ServerName,

    pub server: ST::Config,

    pub management: ManagementConfig,

    pub tls_config: TlsConfig,
    pub node_config: NodeConfig,
    pub event_log: EventLogConfig,

    pub log: LoggingConfig,
}

impl<ST> ServerConfig<ST>
where
    ST: ServerType,
    ST::Config: DeserializeOwned,
{
    /// Load configuration from a file
    pub fn load_file<P: AsRef<Path>>(filename: P) -> Result<Self, anyhow::Error> {
        let mut file = File::open(filename)?;
        let mut config = String::new();
        file.read_to_string(&mut config)?;
        Ok(json5::from_str(&config)?)
    }
}

/// A network server.
///
/// This type contains a [`ReplicatedEventLog`], a [`NetworkNode`], an application-specific
/// server type `ST`, and a management service. It handles communications between them,
/// and co-ordinates shutdown as needed.
pub struct Server<ST> {
    node: Arc<NetworkNode>,
    log: Arc<ReplicatedEventLog>,
    server: Arc<ST>,
    management_config: ManagementConfig,
    tls_data: TlsData,
    remote_command_recv: Mutex<Option<UnboundedReceiver<RemoteServerRequest>>>,
}

/// Saved state of a network server
#[derive(Serialize, Deserialize)]
pub struct ServerState<ST>
where
    ST: ServerType,
{
    node_state: NetworkNodeState,
    log_state: ReplicatedEventLogState,
    server_state: ST::Saved,
}

impl<ST> Server<ST>
where
    ST: ServerType,
{
    /// Construct a server.
    ///
    /// If `bootstrap_config` is `None`, then this function will call out to one of the defined
    /// network peers to retrieve the current network state.
    pub async fn new(
        conf: ServerConfig<ST>,
        server_conf: ST::ProcessedConfig,
        tls_data: TlsData,
        net_config: SyncConfig,
        bootstrap_config: Option<NetworkConfig>,
    ) -> anyhow::Result<Self> {
        let (server_send, server_recv) = unbounded_channel();
        let (history_send, history_recv) = unbounded_channel();
        let (remote_send, remote_recv) = unbounded_channel();

        let policy = StandardPolicyService::new();
        let epoch = EpochId::new(chrono::Utc::now().timestamp());

        let log = Arc::new(ReplicatedEventLog::new(
            conf.server_id,
            &conf.server_name,
            epoch,
            server_send,
            net_config,
            conf.node_config,
            conf.event_log,
        ));

        let network = match bootstrap_config {
            Some(conf) => Network::new(conf),
            None => *log.sync_to_network().await,
        };

        let node = Arc::new(NetworkNode::new(
            conf.server_id,
            epoch,
            conf.server_name,
            network,
            Arc::clone(&log),
            server_recv,
            history_send,
            Some(remote_send),
            policy,
        ));

        let server = Arc::new(
            ST::new(server_conf, &tls_data, Arc::clone(&node), history_recv)
                .context("Could not initialize server")?,
        );

        Ok(Self {
            node,
            log,
            server,
            management_config: conf.management,
            tls_data: conf
                .tls_config
                .load_from_disk()
                .expect("Couldn't load TLS files"),
            remote_command_recv: Mutex::new(Some(remote_recv)),
        })
    }

    /// Run the server, including the log synchronisation, the network state tracking, management
    /// service, and application-specific logic provided by `ST`.
    pub async fn run(&self) -> ShutdownAction {
        let (shutdown_send, shutdown_recv) = broadcast::channel(1);

        let log_task = self.log.start_sync(shutdown_send.subscribe());

        let node = Arc::clone(&self.node);
        let node_task = tokio::spawn(async move {
            node.run(shutdown_recv).await;
        });

        let server = Arc::clone(&self.server);

        let shutdown_recv = shutdown_send.subscribe();
        let server_task = tokio::spawn(async move { server.run(shutdown_recv).await });

        let shutdown_recv = shutdown_send.subscribe();
        let remote_command_recv = self
            .remote_command_recv
            .lock()
            .take()
            .expect("Remote command channel already taken?");
        let server = Arc::clone(&self.server);
        let event_pump_task = tokio::spawn(async move {
            Self::run_event_pump(shutdown_recv, remote_command_recv, server).await
        });

        // The management task will exit on receiving a shutdown command, so just wait for it to finish
        // then propagate the shutdown action
        let action = self.run_management().await;

        shutdown_send
            .send(action.clone())
            .expect("Couldn't signal shutdown");

        event_pump_task.await.expect("Event pump panicked");
        node_task.await.expect("Node task panicked");
        server_task.await.expect("Server task panicked");
        log_task
            .await
            .expect("Log task panicked?")
            .expect("Log task returned error");

        action
    }

    /// Shut down the server, if it is not going to be resumed.
    pub async fn shutdown(self) {
        let server =
            Arc::try_unwrap(self.server).unwrap_or_else(|_| panic!("Couldn't unwrap server"));

        server.shutdown().await;
    }

    async fn run_event_pump(
        mut shutdown: broadcast::Receiver<ShutdownAction>,
        mut remote_commands: UnboundedReceiver<RemoteServerRequest>,
        server: Arc<ST>,
    ) {
        loop {
            tokio::select! {
                _ = shutdown.recv() =>
                {
                    break;
                }
                request = remote_commands.recv() =>
                {
                    if let Some(request) = request
                    {
                        let response = server.handle_remote_command(request.req);
                        if let Err(e) = request.response.send(response)
                        {
                            tracing::error!(?e, "Couldn't send response to remote command");
                        }
                    }
                    else
                    {
                        break;
                    }
                }
            }
        }
    }

    async fn run_management(&self) -> ShutdownAction {
        let (server_shutdown_send, server_shutdown_recv) = oneshot::channel();

        let mut server = management::ManagementServer::start(
            self.management_config.clone(),
            self.tls_data.clone(),
            server_shutdown_recv,
        );

        let shutdown_action = loop {
            if let Some(cmd) = server.recv().await {
                tracing::debug!("Received from management server");
                match cmd {
                    management::ManagementCommand::ServerCommand(scmd) => {
                        let command = &scmd.cmd;
                        tracing::debug!(?command, "Management server command");
                        self.node.handle_management_command(scmd).await;
                    }
                    management::ManagementCommand::Shutdown(action) => {
                        break action;
                    }
                }
            } else {
                tracing::error!("Lost management server, shutting down");
                break ShutdownAction::Shutdown;
            }
        };

        server_shutdown_send
            .send(())
            .expect("Couldn't signal management service to shut down");
        server.wait().await.expect("Management service error");

        shutdown_action
    }

    /// Save the state of the server, including all its component parts, for resumption after a code upgrade.
    pub async fn save(self) -> Result<ServerState<ST>, ServerSaveError> {
        // Order matters here.
        //
        // Arc::try_unwrap will fail if there are any other Arcs still referencing the same object.
        // Because ClientServer holds an Arc to the node, and the node holds an Arc to the log,
        // they have to be saved/deconstructed in this order to get rid of the extra refs so that the
        // next one can be unwrapped.
        let server_state = Arc::try_unwrap(self.server)
            .unwrap_or_else(|_| panic!("Couldn't unwrap server"))
            .save()
            .await?;
        let node_state = Arc::try_unwrap(self.node)
            .unwrap_or_else(|_| panic!("Couldn't unwrap node"))
            .save_state();
        let log_state = Arc::try_unwrap(self.log)
            .unwrap_or_else(|_| panic!("Couldn't unwrap event log"))
            .save_state()
            .map_err(server_type::ServerSaveError::EventLogSaveError)?;

        Ok(ServerState {
            node_state,
            log_state,
            server_state,
        })
    }

    /// Restore from a previously-saved application state.
    pub fn restore_from(
        state: ServerState<ST>,
        net_config: SyncConfig,
        server_config: ServerConfig<ST>,
    ) -> std::io::Result<Self> {
        let (server_send, server_recv) = unbounded_channel();
        let (history_send, history_recv) = unbounded_channel();
        let (remote_send, remote_recv) = unbounded_channel();

        let log = Arc::new(ReplicatedEventLog::restore(
            state.log_state,
            server_send,
            net_config,
            server_config.node_config,
        ));

        let node = Arc::new(NetworkNode::restore_from(
            state.node_state,
            Arc::clone(&log),
            server_recv,
            history_send,
            Some(remote_send),
        )?);

        let server = Arc::new(ST::restore(
            state.server_state,
            Arc::clone(&node),
            history_recv,
        )?);

        Ok(Self {
            node,
            log,
            server,
            management_config: server_config.management,
            tls_data: server_config
                .tls_config
                .load_from_disk()
                .expect("Couldn't load TLS data files"),
            remote_command_recv: Mutex::new(Some(remote_recv)),
        })
    }
}
