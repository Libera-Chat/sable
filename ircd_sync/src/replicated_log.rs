//! A replicated version of [`EventLog`]

use super::*;
use irc_network::{
    event::*,
    id::*,
    ServerName
};
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
    time::sleep,
};
use std::{
    time::Duration,
    collections::HashMap,
};

/// Saved state for a [ReplicatedEventLog], used to save and restore across an upgrade
#[derive(Debug,serde::Serialize,serde::Deserialize)]
pub struct ReplicatedEventLogState
{
    server: (ServerId, EpochId),
    log_state: EventLogState,
    server_tombstones: HashMap<ServerId, (ServerName, EpochId)>,
}

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
    server: (ServerId, EpochId),
    server_tombstones: HashMap<ServerId, (ServerName, EpochId)>,
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
    pub fn new(server_id: ServerId,
               epoch: EpochId,
               server_send: Sender<NetworkMessage>,
               update_recv: Receiver<EventLogUpdate>,
               net_config: NetworkConfig,
               node_config: NodeConfig,
            ) -> Self
    {
        let (log_send, log_recv) = channel(128);
        let (net_send, net_recv) = channel(128);

        Self {
            log: EventLog::new(EventIdGenerator::new(server_id, epoch, 0), Some(log_send)),
            net: Network::new(net_config, node_config, net_send),

            server_send,
            log_recv,
            update_recv,
            network_recv: net_recv,
            server: (server_id, epoch),
            server_tombstones: HashMap::new(),
        }
    }

    /// Construct a `ReplicatedEventLog` from a previously saved state
    /// and given set of configs.
    ///
    /// `state` contains a state object previously returned on completion of
    /// [`sync_task`](Self::sync_task); the other arguments are as for
    /// [`new`](Self::new)
    pub fn restore(state: ReplicatedEventLogState,
               server_send: Sender<NetworkMessage>,
               update_receiver: Receiver<EventLogUpdate>,
               net_config: NetworkConfig,
               node_config: NodeConfig,
            ) -> Self
    {
        let (log_send, log_recv) = channel(128);
        let (net_send, net_recv) = channel(128);

        let log = EventLog::restore(state.log_state, Some(log_send));

        Self {
            log,
            net: Network::new(net_config, node_config, net_send),

            server_send,
            log_recv,
            update_recv: update_receiver,
            network_recv: net_recv,
            server: state.server,
            server_tombstones: state.server_tombstones,
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

    async fn start_sync_to_network(&self, sender: Sender<Request>) -> JoinHandle<()>
    {
        while let Some(peer) = self.net.choose_any_peer()
        {
            tracing::info!("Requesting network state from {:?}", peer);
            let msg = Message { source_server: self.server, content: MessageDetail::GetNetworkState };
            match self.net.send_and_process(peer, msg, sender.clone()).await
            {
                Ok(handle) => return handle,
                Err(_) => sleep(Duration::from_secs(3)).await,
            }
        }
        panic!("No peer available to sync");
    }

    /// Run the main network synchronisation task.
    #[tracing::instrument(skip_all)]
    pub async fn sync_task(mut self, mut shutdown: broadcast::Receiver<ShutdownAction>) -> Result<ReplicatedEventLogState, NetworkError>
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
                        Some(EventLogUpdate::NewEvent(id, detail)) =>
                        {
                            let event = self.log.create(id, detail);
                            tracing::trace!("Server signalled log update: {:?}", event);
                            self.log.add(event.clone());
                            self.net.propagate(&self.message(MessageDetail::NewEvent(event))).await
                        },
                        Some(EventLogUpdate::ServerQuit(name, id, epoch)) =>
                        {
                            self.server_tombstones.insert(id, (name, epoch));
                            self.net.disable_peer(&name.to_string());
                        },
                        Some(EventLogUpdate::ServerJoin(name, id, _epoch)) =>
                        {
                            self.server_tombstones.remove(&id);
                            self.net.enable_peer(&name.to_string());
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
        Ok(ReplicatedEventLogState {
            server: self.server,
            log_state: self.log.save_state(),
            server_tombstones: self.server_tombstones,
        })
    }

    /// Make a [`Message`] originating from this server, for submission to the network
    fn message(&self, content: MessageDetail) -> Message
    {
        Message { source_server: self.server, content }
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
                    response.send(self.message(MessageDetail::GetEvent(missing))).await
                {
                    tracing::error!("Error sending response to network message: {}", e);
                }
            }

            self.log.add(evt.clone());
            if should_propagate
            {
                self.net.propagate(&self.message(MessageDetail::NewEvent(evt))).await;
            }
        }
        is_done
    }

    #[tracing::instrument(skip(self))]
    async fn handle_network_request(&mut self, req: Request)
    {
        // If this is a server we've seen quit, don't accept any events from it
        let (source_id, source_epoch) = &req.message.source_server;
        if let Some((name, tombstone_epoch)) = self.server_tombstones.get(&source_id)
        {
            if tombstone_epoch == source_epoch
            {
                tracing::warn!("Got sync message from tombstoned peer {}, rejecting", name);
                let _ = req.response.send(self.message(MessageDetail::MessageRejected)).await;
                return;
            }
        }

        match req.message.content {
            MessageDetail::NewEvent(evt) =>
            {
                if self.handle_new_event(evt, true, &req.response).await
                {
                    if let Err(e) = req.response.send(self.message(MessageDetail::Done)).await
                    {
                        tracing::error!("Error sending response to network message: {}", e);
                    }
                }
            },
            MessageDetail::BulkEvents(events) =>
            {
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
                    if let Err(e) = req.response.send(self.message(MessageDetail::Done)).await
                    {
                        tracing::error!("Error sending response to network message: {}", e);
                    }
                }
            },
            MessageDetail::SyncRequest(clock) =>
            {
                let new_events: Vec<Event> = self.log.get_since(clock).cloned().collect();

                if let Err(e) =
                    req.response.send(self.message(MessageDetail::BulkEvents(new_events))).await
                {
                    tracing::error!("Error sending response to network message: {}", e);
                }
            },
            MessageDetail::GetEvent(ids) =>
            {
                tracing::debug!("Got request for events {:?}", ids);
                let mut events = Vec::new();

                for id in ids.iter()
                {
                    if let Some(new_event) = self.log.get(id)
                    {
                        events.push(new_event.clone());
                    }
                }
                tracing::debug!("Sending events {:?}", events);
                if let Err(e) =
                    req.response.send(self.message(MessageDetail::BulkEvents(events))).await
                {
                    tracing::error!("Error sending response to network message: {}", e);
                }
            },
            MessageDetail::GetNetworkState =>
            {
                tracing::trace!("Processing get network state request");
                let (send,mut recv) = channel(1);
                if let Err(e) = self.server_send.send(NetworkMessage::ExportNetworkState(send)).await
                {
                    tracing::error!("Error sending network request to server: {}", e);
                }
                if let Some(net) = recv.recv().await
                {
                    if let Err(e) = req.response.send(self.message(MessageDetail::NetworkState(net))).await
                    {
                        tracing::error!("Error sending response to network message: {}", e);
                    }
                }
            },
            MessageDetail::NetworkState(net) =>
            {
                tracing::info!("Got new network state; applying");
                tracing::info!("New event clock is {:?}", net.clock());

                // Set our event clock to the one from the incoming state
                self.log.set_clock(net.clock().clone());

                // Then reset our list of active peers to those that are active in the incoming state
                self.net.disable_all_peers();
                for server in net.servers()
                {
                    self.net.enable_peer(server.name().as_ref());
                }

                if let Err(e) = self.server_send.send(NetworkMessage::ImportNetworkState(net)).await
                {
                    tracing::error!("Error sending network state to server: {}", e);
                }
                if let Err(e) = req.response.send(self.message(MessageDetail::Done)).await
                {
                    tracing::error!("Error sending response to network message: {}", e);
                }
            },
            MessageDetail::MessageRejected =>
            {
                // If the target server rejected one of our sync messages, then they'll continue
                // to reject them until we restart. Stop sending to that server
                tracing::warn!("peer {} rejected our message; disabling", &req.received_from);
                self.net.disable_peer(&req.received_from);
            }
            MessageDetail::Done => {

            }
        }
    }
}
