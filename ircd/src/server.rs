use client_listener::{ListenerCollection, ConnectionEvent};
use sable_ircd::server::ClientServerState;
use sable_network::{node::NetworkNodeState, prelude::config::NetworkConfig, policy::StandardPolicyService, rpc::ShutdownAction};
use tokio::{
    sync::{
        mpsc::UnboundedReceiver,
        oneshot,
    },
};

use crate::config::{ManagementConfig, TlsData, ServerConfig};

use super::*;

use std::sync::Arc;

pub struct Server
{
    node: Arc<NetworkNode>,
    log: Arc<ReplicatedEventLog>,
    server: Arc<ClientServer>,
    management_config: ManagementConfig,
    tls_data: TlsData,
}

#[derive(Serialize,Deserialize)]
pub struct ServerState
{
    node_state: NetworkNodeState,
    log_state: ReplicatedEventLogState,
    server_state: ClientServerState,
}

impl Server
{
    pub async fn new(conf: ServerConfig,
                     net_config: SyncConfig,
                     bootstrap_config: Option<NetworkConfig>,
                     listeners: ListenerCollection,
                     connection_events: UnboundedReceiver<ConnectionEvent>
            ) -> Self
    {
        let (server_send, server_recv) = unbounded_channel();
        let (history_send, history_recv) = unbounded_channel();

        let policy = StandardPolicyService::new();
        let epoch = EpochId::new(chrono::Utc::now().timestamp());

        let log = Arc::new(ReplicatedEventLog::new(conf.server_id, epoch, server_send, net_config, conf.node_config));

        let network = match bootstrap_config {
            Some(conf) => Network::new(conf),
            None => *log.sync_to_network().await
        };

        let node = Arc::new(NetworkNode::new(conf.server_id, epoch, conf.server_name, network, Arc::clone(&log), server_recv, history_send, policy));

        let server = Arc::new(ClientServer::new(Arc::clone(&node), history_recv, listeners, connection_events));

        Self {
            node,
            log,
            server,
            management_config: conf.management,
            tls_data: conf.tls_config.load_from_disk().expect("Couldn't load TLS files")
        }
    }

    pub async fn run(&self) -> ShutdownAction
    {
        let (shutdown_send, shutdown_recv) = broadcast::channel(1);

        let log_task = self.log.start_sync(shutdown_send.subscribe());

        let node = Arc::clone(&self.node);
        let node_task = tokio::spawn(async move {
            node.run(shutdown_recv).await;
        });

        let server = Arc::clone(&self.server);

        let shutdown_recv = shutdown_send.subscribe();
        let server_task = tokio::spawn(async move {
            server.run(shutdown_recv).await
        });

        // The management task will exit on receiving a shutdown command, so just wait for it to finish
        // then propagate the shutdown action
        let action = self.run_management().await;

        shutdown_send.send(action.clone()).expect("Couldn't signal shutdown");

        node_task.await.expect("Node task panicked");
        server_task.await.expect("Server task panicked");
        log_task.await.expect("Log task panicked?").expect("Log task returned error");

        action
    }

    pub async fn save(self) -> ServerState
    {
        // Order matters here.
        //
        // Arc::try_unwrap will fail if there are any other Arcs still referencing the same object.
        // Because ClientServer holds an Arc to the node, and the node holds an Arc to the log,
        // they have to be saved/deconstructed in this order to get rid of the extra refs so that the
        // next one can be unwrapped.
        let server_state = Arc::try_unwrap(self.server)
                            .unwrap_or_else(|_| panic!("Couldn't unwrap server"))
                            .save_state().await
                            .expect("Couldn't save server state");
        let node_state = Arc::try_unwrap(self.node)
                            .unwrap_or_else(|_| panic!("Couldn't unwrap node"))
                            .save_state();
        let log_state = Arc::try_unwrap(self.log)
                            .unwrap_or_else(|_| panic!("Couldn't unwrap event log"))
                            .save_state()
                            .expect("Couldn't save log state");

        ServerState {
            node_state,
            log_state,
            server_state,
        }
    }

    pub fn restore_from(state: ServerState,
                        net_config: SyncConfig,
                        server_config: ServerConfig,
                    ) -> std::io::Result<Self>
    {
        let (server_send, server_recv) = unbounded_channel();
        let (history_send, history_recv) = unbounded_channel();

        let log = Arc::new(ReplicatedEventLog::restore(state.log_state, server_send, net_config, server_config.node_config));

        let node = Arc::new(NetworkNode::restore_from(state.node_state, Arc::clone(&log), server_recv, history_send)?);

        let server = Arc::new(ClientServer::restore_from(state.server_state, Arc::clone(&node), history_recv)?);

        Ok(Self {
            node,
            log,
            server,
            management_config: server_config.management,
            tls_data: server_config.tls_config.load_from_disk().expect("Couldn't load TLS data files")
        })
    }

    pub async fn shutdown(self)
    {
        let server = Arc::try_unwrap(self.server)
                            .unwrap_or_else(|_| panic!("Couldn't unwrap server"));

        server.shutdown().await;
    }

    async fn run_management(&self) -> ShutdownAction
    {
        let (server_shutdown_send, server_shutdown_recv) = oneshot::channel();

        let mut server = management::ManagementServer::start(self.management_config.clone(),
                                                             self.tls_data.clone(),
                                                             server_shutdown_recv);

        let shutdown_action = loop
        {
            if let Some(cmd) = server.recv().await
            {
                match cmd
                {
                    management::ManagementCommand::ServerCommand(scmd) =>
                    {
                        self.server.handle_management_command(scmd).await;
                    }
                    management::ManagementCommand::Shutdown(action) =>
                    {
                        break action;
                    }
                }
            }
            else
            {
                tracing::error!("Lost management server, shutting down");
                break ShutdownAction::Shutdown;
            }
        };

        server_shutdown_send.send(()).expect("Couldn't signal management service to shut down");
        server.wait().await.expect("Management service error");

        shutdown_action

    }
}