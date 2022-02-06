//! A replicated version of [`EventLog`]

use super::*;
use irc_network::event::*;
use irc_network::id::*;
use rpc_protocols::*;

use tokio::{
    sync::mpsc::{
        Sender,
        Receiver,
        channel,
    },
    sync::broadcast,
    select,
    task::JoinHandle,
};

/// A replicated event log.
///
/// `ReplicatedEventLog` wraps [`EventLog`] and adds replication across a
/// network of servers.
pub struct ReplicatedEventLog
{
    log: EventLog,
    net: Network,
    server_send: Sender<NetworkMessage>,
    log_recv: Receiver<Event>,
    update_recv: Receiver<EventLogUpdate>,
    network_recv: Receiver<Request>,
}

impl ReplicatedEventLog
{
    /// Create a new instance.
    ///
    /// ## Arguments
    ///
    /// - `idgen`: Event ID generator
    /// - `server_send`: channel for [`NetworkMessage`]s to be processed
    /// - `update_receiver`: channel for the log to receive [`EventLogUpdate`]
    ///   messages
    /// - `net_config`: global configuration for the gossip network
    /// - `node_config`: configuration specific for this node in the network
    ///
    /// New events will be notified via `server_send` as they become ready for
    /// processing. Events to be emitted by this server should be sent via
    /// `update_receiver` to be created, propagated, and notified back to the
    /// server for processing.
    pub fn new(idgen: EventIdGenerator,
               server_send: Sender<NetworkMessage>,
               update_receiver: Receiver<EventLogUpdate>,
               net_config: NetworkConfig,
               node_config: NodeConfig,
            ) -> Self
    {
        let (log_send, log_recv) = channel(128);
        let (net_send, net_recv) = channel(128);

        Self {
            log: EventLog::new(idgen, Some(log_send)),
            net: Network::new(net_config, node_config, net_send),

            server_send: server_send,
            log_recv: log_recv,
            update_recv: update_receiver,
            network_recv: net_recv,
        }
    }

    /// Construct a `ReplicatedEventLog` from a previously saved state
    /// and given set of configs.
    ///
    /// `state` contains a state object previously returned on completion of
    /// [`sync_task`]; the other arguments are as for [`new`]
    pub fn restore(state: EventLogState,
               server_send: Sender<NetworkMessage>,
               update_receiver: Receiver<EventLogUpdate>,
               net_config: NetworkConfig,
               node_config: NodeConfig,
            ) -> Self
    {
        let (log_send, log_recv) = channel(128);
        let (net_send, net_recv) = channel(128);

        Self {
            log: EventLog::restore(state, Some(log_send)),
            net: Network::new(net_config, node_config, net_send),

            server_send: server_send,
            log_recv: log_recv,
            update_recv: update_receiver,
            network_recv: net_recv,
        }
    }

    /// Run and wait for the initial synchronisation to the network.
    ///
    /// This will choose a peer from the provided network configuration,
    /// request a copy of the current network state from that peer, send it
    /// (via the `server_send` channel provided to the constructor) to be
    /// imported, and update the log's event clock to the current value from
    /// the imported state.
    #[tracing::instrument(skip(self))]
    pub async fn sync_to_network(&mut self)
    {
        let (send, mut recv) = channel(16);
        let handle = self.start_sync_to_network(send).await;

        while let Some(req) = recv.recv().await
        {
            tracing::debug!("Bootstrap message: {:?}", req.message);
            self.handle_network_request(req).await;
        }
        handle.await.expect("Error syncing to network");
    }

    /// Resume syncing from an existing network state. Primarily for use
    /// after a code upgrade.
    pub fn resume_with_state(&mut self, network: &irc_network::Network)
    {
        // We don't actually store any significant state apart from the latest event clock
        self.log.set_clock(network.clock().clone());
    }

    async fn start_sync_to_network(&self, sender: Sender<Request>) -> JoinHandle<()>
    {
        for _ in [0..3]
        {
            if let Some(peer) = self.net.choose_peer()
            {
                tracing::info!("Requesting network state from {:?}", peer);
                let msg = Message::GetNetworkState;
                match self.net.send_and_process(peer, msg, sender.clone()).await
                {
                    Ok(handle) => return handle,
                    Err(_) => continue,
                }
            }
        }
        tokio::spawn(async {})
    }

    /// Run the main network synchronisation task.
    #[tracing::instrument(skip_all)]
    pub async fn sync_task(mut self, mut shutdown: broadcast::Receiver<ShutdownAction>) -> std::io::Result<EventLogState>
    {
        let listen_task = self.net.spawn_listen_task().await?;

        loop {
            tracing::trace!("sync_task loop");
            select! {
                evt = self.log_recv.recv() => {
                    tracing::trace!("...from log_recv");
                    match evt {
                        Some(evt) => {
                            tracing::trace!("Log emitted event: {:?}", evt);

                            if self.server_send.send(NetworkMessage::NewEvent(evt)).await.is_err()
                            {
                                break;
                            }
                        },
                        None => break
                    }
                },
                update = self.update_recv.recv() => {
                    tracing::trace!("...from update_recv");
                    match update {
                        Some(EventLogUpdate::NewEvent(id, detail)) => {
                            let event = self.log.create(id, detail);
                            tracing::trace!("Server signalled log update: {:?}", event);
                            self.log.add(event.clone());
                            self.net.propagate(&Message::NewEvent(event)).await
                        },
                        None => break
                    }
                },
                req = self.network_recv.recv() => {
                    tracing::trace!("...from network_recv: {:?}", req);
                    match req {
                        Some(req) => {
                            self.handle_network_request(req).await;
                        }
                        None => break
                    }
                },
                _ = shutdown.recv() => {
                    break
                }
            }
        }

        self.net.shutdown();
        listen_task.await?;
        Ok(self.log.save_state())
    }

    #[tracing::instrument(skip(self,response))]
    async fn handle_new_event(&mut self, evt: Event, should_propagate: bool, response: &Sender<Message>) -> bool
    {
        let mut is_done = true;
        // Process this event only if we haven't seen it before
        if self.log.get(&evt.id).is_none()
        {
            tracing::trace!("Network sync new event: {:?}", evt);
            // If we're missing any dependencies, ask for them
            if !self.log.has_dependencies_for(&evt)
            {
                is_done = false;
                let missing = self.log.missing_ids_for(&evt.clock);
                tracing::info!("Requesting missing IDs {:?}", missing);
                if let Err(e) =
                    response.send(Message::GetEvent(missing)).await
                {
                    tracing::error!("Error sending response to network message: {}", e);
                }
            }

            self.log.add(evt.clone());
            if should_propagate
            {
                self.net.propagate(&Message::NewEvent(evt)).await;
            }
        }
        is_done
    }

    #[tracing::instrument(skip(self))]
    async fn handle_network_request(&mut self, req: Request)
    {
        match req.message {
            Message::NewEvent(evt) => {
                if self.handle_new_event(evt, true, &req.response).await
                {
                    if let Err(e) = req.response.send(Message::Done).await
                    {
                        tracing::error!("Error sending response to network message: {}", e);
                    }
                }
            },
            Message::BulkEvents(events) => {
                tracing::debug!("Got bulk events: {:?}", events);
                let mut done = true;
                for event in events {
                    // In a bulk sync, don't propagate out again because they're already propagating elsewhere
                    if ! self.handle_new_event(event, false, &req.response).await
                    {
                        done = false;
                    }
                }
                // Send done message only if none of the event processing steps indicated they need more
                if done
                {
                    if let Err(e) = req.response.send(Message::Done).await
                    {
                        tracing::error!("Error sending response to network message: {}", e);
                    }
                }
            },
            Message::SyncRequest(clock) => {
                let new_events: Vec<Event> = self.log.get_since(clock).map(|r| r.clone()).collect();

                if let Err(e) =
                    req.response.send(Message::BulkEvents(new_events)).await
                {
                    tracing::error!("Error sending response to network message: {}", e);
                }
            },
            Message::GetEvent(ids) => {
                tracing::debug!("Got request for events {:?}", ids);
                let mut events = Vec::new();

                for id in ids.iter()
                {
                    if let Some(new_event) = self.log.get(&id)
                    {
                        events.push(new_event.clone());
                    }
                }
                tracing::debug!("Sending events {:?}", events);
                if let Err(e) =
                    req.response.send(Message::BulkEvents(events)).await
                {
                    tracing::error!("Error sending response to network message: {}", e);
                }
            },
            Message::GetNetworkState => {
                tracing::trace!("Processing get network state request");
                let (send,mut recv) = channel(1);
                if let Err(e) = self.server_send.send(NetworkMessage::ExportNetworkState(send)).await
                {
                    tracing::error!("Error sending network request to server: {}", e);
                }
                if let Some(net) = recv.recv().await
                {
                    if let Err(e) = req.response.send(Message::NetworkState(net)).await
                    {
                        tracing::error!("Error sending response to network message: {}", e);
                    }
                }
            },
            Message::NetworkState(net) => {
                tracing::info!("Got new network state; applying");
                tracing::info!("New event clock is {:?}", net.clock());
                self.log.set_clock(net.clock().clone());

                if let Err(e) = self.server_send.send(NetworkMessage::ImportNetworkState(net)).await
                {
                    tracing::error!("Error sending network state to server: {}", e);
                }
                if let Err(e) = req.response.send(Message::Done).await
                {
                    tracing::error!("Error sending response to network message: {}", e);
                }
            },
            Message::Done => {

            }
        }
    }
}
