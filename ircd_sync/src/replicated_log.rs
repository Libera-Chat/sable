use super::*;
use irc_network::event::*;
use irc_network::id::*;
use irc_server::ServerRpcMessage;

use tokio::{
    sync::mpsc::{
        Sender,
        Receiver,
        channel,
    },
    select,
    task::JoinHandle,
};

pub struct ReplicatedEventLog
{
    log: EventLog,
    net: Network,
    server_send: Sender<ServerRpcMessage>,
    log_recv: Receiver<Event>,
    update_recv: Receiver<EventLogUpdate>,
    network_recv: Receiver<Request>,
}

impl ReplicatedEventLog
{
    pub fn new(idgen: EventIdGenerator,
               server_send: Sender<ServerRpcMessage>,
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

    pub async fn sync_to_network(&mut self)
    {
        let (send, mut recv) = channel(16);
        let handle = self.start_sync_to_network(send).await;

        while let Some(req) = recv.recv().await
        {
            log::debug!("Bootstrap message: {:?}", req.message);
            self.handle_network_request(req).await;
        }
        handle.await.expect("Error syncing to network");
    }

    async fn start_sync_to_network(&self, sender: Sender<Request>) -> JoinHandle<()>
    {
        for _ in [0..3]
        {
            if let Some(peer) = self.net.choose_peer()
            {
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

    pub async fn sync_task(mut self)
    {
        self.net.spawn_listen_task().await;

        loop {
            log::trace!("sync_task loop");
            select! {
                evt = self.log_recv.recv() => {
                    log::trace!("...from log_recv");
                    match evt {
                        Some(evt) => {
                            log::trace!("Log emitted event: {:?}", evt);

                            if self.server_send.send(ServerRpcMessage::NewEvent(evt)).await.is_err()
                            {
                                break;
                            }
                        },
                        None => break
                    }
                },
                update = self.update_recv.recv() => {
                    log::trace!("...from update_recv");
                    match update {
                        Some(EventLogUpdate::NewEvent(id, detail)) => {
                            let event = self.log.create(id, detail);
                            log::trace!("Server signalled log update: {:?}", event);
                            self.log.add(event.clone());
                            self.net.propagate(&Message::NewEvent(event)).await
                        },
                        Some(EventLogUpdate::EpochUpdate(new_epoch)) => {
                            log::debug!("Server signalled epoch update: {:?}", new_epoch);
                            self.log.set_epoch(new_epoch);
                        },
                        None => break
                    }
                },
                req = self.network_recv.recv() => {
                    log::trace!("...from network_recv: {:?}", req);
                    match req {
                        Some(req) => {
                            self.handle_network_request(req).await;
                        }
                        None => break
                    }
                }
            }
        }
    }

    async fn handle_new_event(&mut self, evt: Event, should_propagate: bool, response: &Sender<Message>) -> bool
    {
        let mut is_done = true;
        // Process this event only if we haven't seen it before
        if self.log.get(&evt.id).is_none()
        {
            log::trace!("Network sync new event: {:?}", evt);
            // If we're missing any dependencies, ask for them
            if !self.log.has_dependencies_for(&evt)
            {
                is_done = false;
                let missing = self.log.missing_ids_for(&evt.clock);
                log::info!("Requesting missing IDs {:?}", missing);
                if let Err(e) = 
                    response.send(Message::GetEvent(missing)).await
                {
                    log::error!("Error sending response to network message: {}", e);
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

    async fn handle_network_request(&mut self, req: Request)
    {
        match req.message {
            Message::NewEvent(evt) => {
                if self.handle_new_event(evt, true, &req.response).await
                {
                    if let Err(e) = req.response.send(Message::Done).await
                    {
                        log::error!("Error sending response to network message: {}", e);
                    }
                }
            },
            Message::BulkEvents(events) => {
                log::debug!("Got bulk events: {:?}", events);
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
                        log::error!("Error sending response to network message: {}", e);
                    }
                }
            },
            Message::SyncRequest(clock) => {
                let new_events: Vec<Event> = self.log.get_since(clock).map(|r| r.clone()).collect();

                if let Err(e) = 
                    req.response.send(Message::BulkEvents(new_events)).await
                {
                    log::error!("Error sending response to network message: {}", e);
                }
            },
            Message::GetEvent(ids) => {
                log::debug!("Got request for events {:?}", ids);
                let mut events = Vec::new();

                for id in ids.iter()
                {
                    if let Some(new_event) = self.log.get(&id)
                    {
                        events.push(new_event.clone());
                    }
                }
                log::debug!("Sending events {:?}", events);
                if let Err(e) = 
                    req.response.send(Message::BulkEvents(events)).await
                {
                    log::error!("Error sending response to network message: {}", e);
                }
            },
            Message::GetNetworkState => {
                log::trace!("Processing get network state request");
                let (send,mut recv) = channel(1);
                if let Err(e) = self.server_send.send(ServerRpcMessage::ExportNetworkState(send)).await
                {
                    log::error!("Error sending network request to server: {}", e);
                }
                if let Some(net) = recv.recv().await
                {
                    if let Err(e) = req.response.send(Message::NetworkState(net)).await
                    {
                        log::error!("Error sending response to network message: {}", e);
                    }
                }
            },
            Message::NetworkState(net) => {
                log::debug!("Got new network state; applying");
                log::debug!("New event clock is {:?}", net.clock());
                self.log.set_clock(net.clock().clone());

                if let Err(e) = self.server_send.send(ServerRpcMessage::ImportNetworkState(net)).await
                {
                    log::error!("Error sending network state to server: {}", e);
                }
                if let Err(e) = req.response.send(Message::Done).await
                {
                    log::error!("Error sending response to network message: {}", e);
                }
            },
            Message::Done => {

            }
        }
    }
}
