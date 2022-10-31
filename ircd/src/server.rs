use client_listener::{ListenerCollection, ConnectionEvent};
use sable_ircd::server::{ClientServerState, ServerManagementCommand};
use sable_network::{node::NetworkNodeState, prelude::config::NetworkConfig, policy::StandardPolicyService, rpc::ShutdownAction};
use tokio::sync::mpsc::{Receiver, UnboundedReceiver};

use super::*;

use std::sync::Arc;

pub struct Server
{
    node: Arc<NetworkNode>,
    log: Arc<ReplicatedEventLog>,
    server: Arc<ClientServer>,
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
    pub async fn new(id: ServerId,
                     epoch: EpochId,
                     name: ServerName,
                     net_config: SyncConfig,
                     node_config: NodeConfig,
                     bootstrap_config: Option<NetworkConfig>,
                     listeners: ListenerCollection,
                     connection_events: UnboundedReceiver<ConnectionEvent>
            ) -> Self
    {
        let (server_send, server_recv) = unbounded_channel();
        let (history_send, history_recv) = unbounded_channel();

        let policy = StandardPolicyService::new();

        let log = Arc::new(ReplicatedEventLog::new(id, epoch, server_send, net_config, node_config));

        let network = match bootstrap_config {
            Some(conf) => Network::new(conf),
            None => *log.sync_to_network().await
        };

        let node = Arc::new(NetworkNode::new(id, epoch, name, network, Arc::clone(&log), server_recv, history_send, policy));

        let server = Arc::new(ClientServer::new(Arc::clone(&node), history_recv, listeners, connection_events));

        Self {
            node,
            log,
            server,
        }
    }

    pub async fn run(&self, management: Receiver<ServerManagementCommand>, shutdown: oneshot::Receiver<ShutdownAction>) -> ShutdownAction
    {
        let (log_shutdown_send, log_shutdown_recv) = oneshot::channel();
        let (server_shutdown_send, server_shutdown_recv) = oneshot::channel();
        let (node_shutdown_send, node_shutdown_recv) = oneshot::channel();

        let log_task = self.log.start_sync(log_shutdown_recv);

        let node = Arc::clone(&self.node);
        let node_task = tokio::spawn(async move {
            node.run(node_shutdown_recv).await;
        });

        let server = Arc::clone(&self.server);

        let server_task = tokio::spawn(async move {
            server.run(management, server_shutdown_recv).await
        });

        let action = shutdown.await.expect("Shutdown channel error");

        node_shutdown_send.send(action.clone()).expect("Couldn't signal node to shutdown");
        node_task.await.expect("Node task panicked");

        server_shutdown_send.send(action.clone()).expect("Couldn't signal server to shutdown");
        server_task.await.expect("Server task panicked");

        log_shutdown_send.send(action.clone()).expect("Couldn't signal log to shutdown");
        log_task.await.expect("Log task panicked?").expect("Log task returned error");

        action
    }

    pub async fn save(self) -> std::io::Result<ServerState>
    {
        // Order matters here.
        //
        // Arc::try_unwrap will fail if there are any other Arcs still referencing the same object.
        // Because ClientServer holds an Arc to the node, and the node holds an Arc to the log,
        // they have to be saved/deconstructed in this order to get rid of the extra refs so that the
        // next one can be unwrapped.
        let server_state = Arc::try_unwrap(self.server)
                            .unwrap_or_else(|_| panic!("Couldn't unwrap server"))
                            .save_state().await?;
        let node_state = Arc::try_unwrap(self.node)
                            .unwrap_or_else(|_| panic!("Couldn't unwrap node"))
                            .save_state();
        let log_state = Arc::try_unwrap(self.log)
                            .unwrap_or_else(|_| panic!("Couldn't unwrap event log"))
                            .save_state()
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, Box::new(e)))?;

        Ok(ServerState {
            node_state,
            log_state,
            server_state,
        })
    }

    pub fn restore_from(state: ServerState,
                        net_config: SyncConfig,
                        node_config: NodeConfig,
                    ) -> std::io::Result<Self>
    {
        let (server_send, server_recv) = unbounded_channel();
        let (history_send, history_recv) = unbounded_channel();

        let log = Arc::new(ReplicatedEventLog::restore(state.log_state, server_send, net_config, node_config));

        let node = Arc::new(NetworkNode::restore_from(state.node_state, Arc::clone(&log), server_recv, history_send)?);

        let server = Arc::new(ClientServer::restore_from(state.server_state, Arc::clone(&node), history_recv)?);

        Ok(Self {
            node,
            log,
            server,
        })
    }

    pub async fn shutdown(self)
    {
        let server = Arc::try_unwrap(self.server)
                            .unwrap_or_else(|_| panic!("Couldn't unwrap server"));

        server.shutdown().await;
    }
}