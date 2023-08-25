//! A replicated version of [`EventLog`]

use crate::prelude::*;
use crate::rpc::*;

use std::{collections::HashMap, sync::Arc, sync::RwLock, time::Duration};
use tokio::{
    select,
    sync::{
        broadcast,
        mpsc::{channel, unbounded_channel, Sender, UnboundedReceiver, UnboundedSender},
        oneshot, Mutex,
    },
    task::JoinHandle,
    time::sleep,
};

use thiserror::Error;

use super::message::TargetedMessage;
use super::network::NetworkResult;

#[derive(Debug, Error)]
pub enum EventLogSaveError {
    #[error("Sync task is still running")]
    TaskStillRunning,
    #[error("{0}")]
    InternalError(&'static str),
    #[error("Unknown error: {0}")]
    UnknownError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Saved state for a [ReplicatedEventLog], used to save and restore across an upgrade
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ReplicatedEventLogState {
    server: (ServerId, EpochId),
    log_state: EventLogState,
    server_tombstones: HashMap<ServerId, (ServerName, EpochId)>,
    network_state: GossipNetworkState,
}

/// A replicated event log.
///
/// `ReplicatedEventLog` wraps [`EventLog`] and adds replication across a
/// network of servers.
pub struct ReplicatedEventLog {
    shared_state: Arc<SharedState>,
    task_state: Arc<Mutex<TaskState>>,
    new_event_send: UnboundedSender<EventLogMessage>,
    net: Arc<GossipNetwork>,
}

struct SharedState {
    server: (ServerId, EpochId),
    server_tombstones: RwLock<HashMap<ServerId, (ServerName, EpochId)>>,
    log: RwLock<EventLog>,
}

struct TaskState {
    net: Arc<GossipNetwork>,
    new_event_recv: UnboundedReceiver<EventLogMessage>,
    network_recv: UnboundedReceiver<Request>,
    log_recv: UnboundedReceiver<Event>,
    server_send: UnboundedSender<NetworkMessage>,
    shared_state: Arc<SharedState>,
}

#[derive(Debug)]
enum EventLogMessage {
    NewEvent(ObjectId, EventDetails),
    TargetedMessage(TargetedMessage, oneshot::Sender<RemoteServerResponse>),
}

impl ReplicatedEventLog {
    /// Create a new instance.
    ///
    /// ## Arguments
    ///
    /// - `server_id`: Our server ID, for use in generating event IDs
    /// - `epoch`: Our epoch ID, for use in generating event IDs
    /// - `server_send`: channel for [`NetworkMessage`]s to be processed
    /// - `net_config`: global configuration for the gossip network
    /// - `node_config`: configuration specific for this node in the network
    ///
    /// New events will be notified via `server_send` as they become ready for
    /// processing. Events to be emitted by this server should be sent via
    /// `update_receiver` to be created, propagated, and notified back to the
    /// server for processing.
    pub fn new(
        server_id: ServerId,
        epoch: EpochId,
        server_send: UnboundedSender<NetworkMessage>,
        net_config: SyncConfig,
        node_config: NodeConfig,
    ) -> Self {
        let (log_send, log_recv) = unbounded_channel();
        let (net_send, net_recv) = unbounded_channel();
        let (new_event_send, new_event_recv) = unbounded_channel();

        let net = Arc::new(GossipNetwork::new(net_config, node_config, net_send));

        let shared_state = Arc::new(SharedState {
            server: (server_id, epoch),
            server_tombstones: RwLock::new(HashMap::new()),
            log: RwLock::new(EventLog::new(
                EventIdGenerator::new(server_id, epoch, 0),
                Some(log_send),
            )),
        });

        let task_state = Arc::new(Mutex::new(TaskState {
            net: Arc::clone(&net),
            new_event_recv,
            network_recv: net_recv,
            log_recv,
            server_send,
            shared_state: Arc::clone(&shared_state),
        }));

        Self {
            shared_state,
            task_state,
            net,
            new_event_send,
        }
    }

    /// Construct a `ReplicatedEventLog` from a previously saved state
    /// and given set of configs.
    ///
    /// `state` contains a state object previously returned from
    /// [`save_state`](Self::save_state); the other arguments are as for
    /// [`new`](Self::new)
    pub fn restore(
        state: ReplicatedEventLogState,
        server_send: UnboundedSender<NetworkMessage>,
        net_config: SyncConfig,
        node_config: NodeConfig,
    ) -> Self {
        let (log_send, log_recv) = unbounded_channel();
        let (net_send, net_recv) = unbounded_channel();
        let (new_event_send, new_event_recv) = unbounded_channel();

        let net = Arc::new(GossipNetwork::restore(
            state.network_state,
            net_config,
            node_config,
            net_send,
        ));

        let shared_state = Arc::new(SharedState {
            server: state.server,
            server_tombstones: RwLock::new(state.server_tombstones),
            log: RwLock::new(EventLog::restore(state.log_state, Some(log_send))),
        });

        let task_state = Arc::new(Mutex::new(TaskState {
            net: Arc::clone(&net),
            new_event_recv,
            network_recv: net_recv,
            log_recv,
            server_send,
            shared_state: Arc::clone(&shared_state),
        }));

        Self {
            shared_state,
            task_state,
            net,
            new_event_send,
        }
    }

    /// Create and propagate a new event.
    ///
    /// Arguments are the target object ID, and the event detail.
    pub fn create_event(&self, target: ObjectId, detail: EventDetails) {
        self.new_event_send
            .send(EventLogMessage::NewEvent(target, detail))
            .expect("Failed to submit new event for creation");
    }

    /// Disable a given server for sync purposes. This should be called when
    /// a server is seen to leave the network, and will prevent any incoming sync
    /// messages originating from this server and epoch being accepted, as well as
    /// preventing any new outgoing sync messages from being sent to it.
    pub fn disable_server(&self, name: ServerName, id: ServerId, epoch: EpochId) {
        self.shared_state
            .server_tombstones
            .write()
            .unwrap()
            .insert(id, (name, epoch));
        self.net.disable_peer(name.as_ref());
    }

    /// Enable a server for sync purposes. This should be called when a server is
    /// seen to join the network, and will enable both inbound and outbound
    /// sync messages for the given server.
    pub fn enable_server(&self, name: ServerName, id: ServerId) {
        self.shared_state
            .server_tombstones
            .write()
            .unwrap()
            .remove(&id);
        self.net.enable_peer(name.as_ref());
    }

    /// Send a request to another server in the network, and wait for the response
    pub async fn send_remote_request(
        &self,
        target: ServerName,
        request: RemoteServerRequestType,
    ) -> Result<RemoteServerResponse, NetworkError> {
        let (sender, receiver) = oneshot::channel();

        let targeted_message = TargetedMessage {
            source: self.net.me().name.clone(),
            target: target.to_string(),
            via: Vec::new(),
            content: request,
        };
        self.new_event_send
            .send(EventLogMessage::TargetedMessage(targeted_message, sender))
            .expect("Couldn't send to event log task");

        // If this fails, it's because the Sender dropped, almost certainly because the sync task
        // gave up on waiting for a response
        receiver
            .await
            .map_err(|_| NetworkError::InternalError("channel receive error".to_string()))
    }

    /// Run and wait for the initial synchronisation to the network.
    ///
    /// This will choose a peer from the provided network configuration,
    /// request a copy of the current network state from that peer, return
    /// it, and update the log's event clock to the current value from
    /// the imported state.
    #[tracing::instrument(skip(self))]
    pub async fn sync_to_network(&self) -> Box<crate::network::Network> {
        let net = 'outer: loop {
            let (send, mut recv) = unbounded_channel();
            let handle = self.start_sync_to_network(send).await;

            while let Some(req) = recv.recv().await {
                tracing::debug!("Bootstrap message: {:?}", req.message);
                match req.message.content {
                    MessageDetail::NetworkState(net) => {
                        break 'outer net;
                    }
                    _ => {
                        continue;
                    }
                }
            }
            handle
                .await
                .expect("Error syncing to network")
                .expect("Error syncing to network");
        };

        for server in net.servers() {
            self.enable_server(*server.name(), server.id());
        }
        self.shared_state
            .log
            .write()
            .expect("event log lock is poisoned?")
            .set_clock(net.clock().clone());

        net
    }

    async fn start_sync_to_network(
        &self,
        sender: UnboundedSender<Request>,
    ) -> JoinHandle<NetworkResult> {
        let mut attempts = 0;
        while let Some(peer) = self.net.choose_any_peer() {
            attempts += 1;
            if attempts >= 3 {
                tracing::info!(
                    "Requesting network state from {:?} (attempt #{}).",
                    peer,
                    attempts
                );
                if attempts % 5 == 3 {
                    tracing::warn!("Make sure at least one node in your network is started and reachable. If this is the first (or only) node, you must provide the --bootstrap-network option.");
                }
            } else {
                tracing::debug!("Requesting network state from {:?}", peer);
            }
            let msg = Message {
                source_server: self.shared_state.server,
                content: MessageDetail::GetNetworkState,
            };
            match self.net.send_and_process(peer, msg, sender.clone()).await {
                Ok(handle) => return handle,
                Err(_) => sleep(Duration::from_secs(3)).await,
            }
        }
        panic!("No peer available to sync. This probably means you are running a single-node sable_ircd and did not pass the --bootstrap-network option.");
    }

    pub fn start_sync(
        &self,
        shutdown: broadcast::Receiver<ShutdownAction>,
    ) -> JoinHandle<Result<(), NetworkError>> {
        let task_state = Arc::clone(&self.task_state);
        tokio::spawn(async move { task_state.lock().await.sync_task(shutdown).await })
    }

    pub fn event_log(&self) -> std::sync::RwLockReadGuard<EventLog> {
        self.shared_state.log.read().unwrap()
    }

    pub fn save_state(self) -> Result<ReplicatedEventLogState, EventLogSaveError> {
        // This set of structs takes a bit of untangling to deconstruct.
        // First, extract the task state from the mutex. If this fails (because there's
        // another reference to the Arc), it'll be because the sync task is still running,
        // so the Err return can indicate that.
        let task_state = Arc::try_unwrap(self.task_state)
            .map_err(|_| EventLogSaveError::TaskStillRunning)?
            .into_inner();

        // Now drop the task_state's reference to the shared_state, so that we hold the only one
        drop(task_state.shared_state);
        // ...and unwrap it.
        let shared_state = Arc::try_unwrap(self.shared_state)
            .map_err(|_| EventLogSaveError::InternalError("Couldn't unwrap shared state"))?;

        // Extract the tombstones from the RwLock so we don't have to clone it
        let server_tombstones = shared_state.server_tombstones.into_inner().unwrap();
        // And the same for the log
        let log = shared_state.log.into_inner().unwrap();

        Ok(ReplicatedEventLogState {
            server: shared_state.server,
            log_state: log.save_state(),
            server_tombstones,
            network_state: self.net.save_state(),
        })
    }
}

impl TaskState {
    /// Run the main network synchronisation task.
    #[tracing::instrument(skip_all)]
    async fn sync_task(
        &mut self,
        mut shutdown: broadcast::Receiver<ShutdownAction>,
    ) -> Result<(), NetworkError> {
        let listen_task = self.net.spawn_listen_task().await?;

        loop {
            tracing::trace!("sync_task loop");
            select! {
                evt = self.log_recv.recv() => {
                    tracing::trace!("...from log_recv");
                    match evt {
                        Some(evt) => {
                            tracing::trace!("Log emitted event: {:?}", evt);

                            if self.server_send.send(NetworkMessage::NewEvent(evt)).is_err()
                            {
                                break;
                            }
                        },
                        None => break
                    }
                },
                update = self.new_event_recv.recv() => {
                    tracing::trace!("...from update_recv");
                    match update {
                        Some(EventLogMessage::NewEvent(id, detail)) =>
                        {
                            let event = {
                                let mut log = self.shared_state.log.write().unwrap();
                                let event = log.create(id, detail);
                                tracing::trace!("Server signalled log update: {:?}", event);
                                log.add(event.clone());
                                event
                            };
                            self.net.propagate(&self.message(MessageDetail::NewEvent(event))).await
                        },
                        Some(EventLogMessage::TargetedMessage(message, sender)) =>
                        {
                            let timeout = tokio::time::timeout(Duration::from_secs(5),
                                                               self.send_targeted_message(message));

                            match timeout.await
                            {
                                Ok(Ok(response)) =>
                                {
                                    // If the receiver hung up, the response isn't relevant so don't do anything
                                    let _ = sender.send(response);
                                }
                                Ok(Err(e)) =>
                                {
                                    tracing::error!("Error sending out remote server message: {}", e);
                                    // As above
                                    let _ = sender.send(RemoteServerResponse::Error("Network error".to_string()));
                                }
                                Err(_) =>
                                {
                                    // If we get into this branch it's because the timeout timed out
                                    let _ = sender.send(RemoteServerResponse::Error("Response timeout".to_string()));
                                }
                            }
                        }
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
        Ok(())
    }

    /// Make a [`Message`] originating from this server, for submission to the network
    fn message(&self, content: MessageDetail) -> Message {
        Message {
            source_server: self.shared_state.server,
            content,
        }
    }

    #[tracing::instrument(skip(self, response))]
    async fn handle_new_event(
        &mut self,
        evt: Event,
        mut should_propagate: bool,
        response: &Sender<Message>,
    ) -> bool {
        let mut is_done = true;
        // Calling reserve() here means we don't need to `await` the send operation while holding the lock on `log`
        let response = response.reserve().await;

        // Anonymous scope here ensures that the lock guard is dropped before calling `propagate` below
        {
            let mut log = self.shared_state.log.write().unwrap();

            // Process this event only if we haven't seen it before
            if log.get(&evt.id).is_none() {
                tracing::trace!("Network sync new event: {:?}", evt);

                log.add(evt.clone());

                // If we're missing any dependencies, ask for them
                if !log.has_dependencies_for(&evt) {
                    is_done = false;
                    let missing = log.missing_ids_for(&evt.clock);
                    tracing::debug!("Requesting missing IDs {:?}", missing);
                    match response {
                        Ok(r) => {
                            r.send(self.message(MessageDetail::GetEvent(missing)));
                        }
                        Err(e) => {
                            tracing::error!("Error sending response to network message: {}", e);
                        }
                    }
                }
            } else {
                should_propagate = false;
            }
        }

        if should_propagate {
            self.net
                .propagate(&self.message(MessageDetail::NewEvent(evt)))
                .await;
        }

        is_done
    }

    fn server_is_tombstoned(&self, id: &ServerId, epoch: &EpochId) -> Option<ServerName> {
        if let Some((name, tombstone_epoch)) =
            self.shared_state.server_tombstones.read().unwrap().get(id)
        {
            if tombstone_epoch == epoch {
                return Some(*name);
            }
        }
        None
    }

    async fn send_targeted_message(
        &self,
        mut detail: TargetedMessage,
    ) -> Result<RemoteServerResponse, NetworkError> {
        let (sender, mut receiver) = unbounded_channel();

        // First, try to send directly to the target
        if let Some(target) = self.net.find_peer(detail.target.as_ref()) {
            tracing::debug!(?target, ?detail, "Found target peer, sending message");

            // If this succeeds, then we could connect successfully to the target server, so any error that occurs
            // later in the process won't be solved by re-routing
            let send_result = self
                .net
                .send_and_process(
                    target,
                    self.message(MessageDetail::TargetedMessage(detail.clone())),
                    sender.clone(),
                )
                .await;

            tracing::debug!(?send_result, "Got send result");

            if send_result.is_ok() {
                while let Some(response) = receiver.recv().await {
                    tracing::debug!(?response, "Got targeted message response");
                    if let MessageDetail::TargetedMessageResponse(resp) = response.message.content {
                        return Ok(resp);
                    }
                }
                // The other end sent back something that's not the response we were expecting. Raise it as an internal error
                return Err(NetworkError::InternalError(
                    "Unexpected response type from targeted message".to_string(),
                ));
            }
        }

        // If the above didn't work, then pick another server (that hasn't already seen it)
        // to try to send it along

        // Make sure it doesn't come back to us
        detail.via.push(self.net.me().name.clone());

        // Keep going until we succeed in sending it somewhere
        while let Some(peer) = self.net.choose_peer_except(&detail.via) {
            // If this succeeds, then we could connect successfully to the target server, so any error that occurs
            // later in the process won't be solved by re-routing
            if self
                .net
                .send_and_process(
                    peer,
                    self.message(MessageDetail::TargetedMessage(detail.clone())),
                    sender.clone(),
                )
                .await
                .is_ok()
            {
                while let Some(response) = receiver.recv().await {
                    if let MessageDetail::TargetedMessageResponse(resp) = response.message.content {
                        return Ok(resp);
                    }
                }
                // The other end sent back something that's not the response we were expecting. Raise it as an internal error
                return Err(NetworkError::InternalError(
                    "Unexpected response type from targeted message".to_string(),
                ));
            }
        }

        Err(NetworkError::InternalError(
            "Ran out of potential peers to route targeted message".to_string(),
        ))
    }

    #[tracing::instrument(skip(self))]
    async fn handle_network_request(&mut self, req: Request) {
        // If this is a server we've seen quit, don't accept any events from it
        let (source_id, source_epoch) = &req.message.source_server;
        if let Some(name) = self.server_is_tombstoned(source_id, source_epoch) {
            tracing::warn!("Got sync message from tombstoned peer {}, rejecting", name);
            let _ = req
                .response
                .send(self.message(MessageDetail::MessageRejected))
                .await;
            return;
        }

        match req.message.content {
            MessageDetail::NewEvent(evt) => {
                if self.handle_new_event(evt, true, &req.response).await {
                    if let Err(e) = req.response.send(self.message(MessageDetail::Done)).await {
                        tracing::error!("Error sending response to network message: {}", e);
                    }
                }
            }
            MessageDetail::BulkEvents(events) => {
                tracing::debug!("Got bulk events: {:?}", events);
                let mut done = true;
                for event in events {
                    // In a bulk sync, don't propagate out again because they're already propagating elsewhere
                    if !self.handle_new_event(event, false, &req.response).await {
                        done = false;
                    }
                }
                // Send done message only if none of the event processing steps indicated they need more
                if done {
                    if let Err(e) = req.response.send(self.message(MessageDetail::Done)).await {
                        tracing::error!("Error sending response to network message: {}", e);
                    }
                }
            }
            MessageDetail::SyncRequest(clock) => {
                let new_events: Vec<Event> = self
                    .shared_state
                    .log
                    .read()
                    .unwrap()
                    .get_since(clock)
                    .cloned()
                    .collect();

                if let Err(e) = req
                    .response
                    .send(self.message(MessageDetail::BulkEvents(new_events)))
                    .await
                {
                    tracing::error!("Error sending response to network message: {}", e);
                }
            }
            MessageDetail::GetEvent(ids) => {
                tracing::debug!("Got request for events {:?}", ids);
                let mut events = Vec::new();

                for id in ids.iter() {
                    if let Some(new_event) = self.shared_state.log.read().unwrap().get(id) {
                        events.push(new_event.clone());
                    }
                }
                tracing::debug!("Sending events {:?}", events);
                if let Err(e) = req
                    .response
                    .send(self.message(MessageDetail::BulkEvents(events)))
                    .await
                {
                    tracing::error!("Error sending response to network message: {}", e);
                }
            }
            MessageDetail::GetNetworkState => {
                tracing::trace!("Processing get network state request");
                let (send, mut recv) = channel(1);
                if let Err(e) = self
                    .server_send
                    .send(NetworkMessage::ExportNetworkState(send))
                {
                    tracing::error!("Error sending network request to server: {}", e);
                }
                if let Some(net) = recv.recv().await {
                    if let Err(e) = req
                        .response
                        .send(self.message(MessageDetail::NetworkState(net)))
                        .await
                    {
                        tracing::error!("Error sending response to network message: {}", e);
                    }
                }
            }
            MessageDetail::NetworkState(net) => {
                tracing::debug!("Got new network state; applying");
                tracing::debug!("New event clock is {:?}", net.clock());

                {
                    // Set our event clock to the one from the incoming state
                    self.shared_state
                        .log
                        .write()
                        .unwrap()
                        .set_clock(net.clock().clone());
                }

                // Then reset our list of active peers to those that are active in the incoming state
                for server in net.servers() {
                    self.net.enable_peer(server.name().as_ref());
                }

                if let Err(e) = self
                    .server_send
                    .send(NetworkMessage::ImportNetworkState(net))
                {
                    tracing::error!("Error sending network state to server: {}", e);
                }
                if let Err(e) = req.response.send(self.message(MessageDetail::Done)).await {
                    tracing::error!("Error sending response to network message: {}", e);
                }
            }
            MessageDetail::TargetedMessage(detail) => {
                let response = if detail.target == self.net.me().name {
                    // We're the target. Handle it
                    let (sender, receiver) = oneshot::channel();
                    let request = RemoteServerRequest {
                        req: detail.content,
                        response: sender,
                    };

                    if let Err(e) = self
                        .server_send
                        .send(NetworkMessage::RemoteServerRequest(request))
                    {
                        tracing::error!("Error sending request to server task: {}", e);
                        Ok(RemoteServerResponse::Error(
                            "Couldn't send request to server task".to_string(),
                        ))
                    } else {
                        receiver.await.map_err(|_| {
                            NetworkError::InternalError("Couldn't send to server task".to_string())
                        })
                    }
                } else {
                    // We're not the target. Pass it along
                    self.send_targeted_message(detail).await
                };

                match response {
                    Ok(response) => {
                        // This will only fail if the receiver hung up, in which case the response isn't relevant any more anyway
                        let _ = req
                            .response
                            .send(self.message(MessageDetail::TargetedMessageResponse(response)))
                            .await;
                    }
                    Err(e) => {
                        tracing::error!("Error handling targeted message: {}", e);
                    }
                }
            }
            MessageDetail::MessageRejected => {
                // If the target server rejected one of our sync messages, then they'll continue
                // to reject them until we restart. Stop sending to that server
                tracing::warn!(
                    "peer {} rejected our message; disabling",
                    &req.received_from
                );
                self.net.disable_peer(&req.received_from);
            }
            MessageDetail::Done | MessageDetail::TargetedMessageResponse(_) => {
                // These are only used in responses, so nothing to do here
            }
        }
    }
}
