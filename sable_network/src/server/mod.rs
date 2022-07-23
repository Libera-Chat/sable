use super::*;
use crate::utils::*;
use crate::prelude::*;
use crate::rpc::*;
use crate::network::event::*;
use crate::sync::ReplicatedEventLog;
use crate::rpc::NetworkMessage;

use crate::policy::PolicyService;
use crate::saveable::Saveable;

use parking_lot::RwLockReadGuard;
use tokio::{
    sync::{
        mpsc::{
            UnboundedSender,
            UnboundedReceiver,
        },
        oneshot,
        Mutex,
    },
    time,
    select,
};

use std::{
    sync::Arc,
    time::Duration,
};

use parking_lot::{
    RwLock,
};

pub type NetworkReadGuard<'a> = parking_lot::RwLockReadGuard<'a, Network>;

mod pings;
mod update_receiver;
mod history;

mod upgrade;
pub use upgrade::ServerState;

/// An IRC client server.
pub struct Server<Policy = crate::policy::StandardPolicyService>
    where Policy: PolicyService
{
    my_id: ServerId,
    name: ServerName,
    version: String,
    net: RwLock<Network>,
    event_log: Arc<ReplicatedEventLog>,
    epoch: EpochId,
    id_generator: ObjectIdGenerator,
    // This needs to be a tokio mutex because we hold it for the duration of `run()`, which awaits a lot
    rpc_receiver: tokio::sync::Mutex<UnboundedReceiver<NetworkMessage>>,
    history_log: RwLock<NetworkHistoryLog>,
    subscriber: UnboundedSender<NetworkHistoryUpdate>,
    policy_service: Policy,
}


impl<Policy: crate::policy::PolicyService> Server<Policy>
{
    /// Construct a server.
    ///
    /// Arguments:
    /// - `id`: This server's ID
    /// - `epoch`: The epoch ID to be used for ID generation. This must be unique
    ///   across all invocations with the same server ID, and should be the same
    ///   as the epoch ID provided to the event log
    /// - `name`: The server's name
    /// - `net`: the initial network state, either received from the network via initial
    ///    sync (in normal operation) or empty (if bootstrapping)
    /// - `event_log`: A `ReplicatedEventLog` that syncs to the network
    /// - `rpc_receiver`: channel to receive messages from the network synchronisation.
    ///   Should be shared with the `ReplicatedEventLog`.
    /// - `state_change_sender`: channel to send out network state changes for consumption
    ///
    pub fn new(id: ServerId,
               epoch: EpochId,
               name: ServerName,
               net: Network,
               event_log: Arc<ReplicatedEventLog>,
               rpc_receiver: UnboundedReceiver<NetworkMessage>,
               subscriber: UnboundedSender<NetworkHistoryUpdate>,
               policy_service: Policy,
            ) -> Self
    {
        if cfg!(feature = "debug") && !net.config().debug_mode
        {
            panic!("Server is built with debug code but network has debug disabled")
        }

        Self {
            my_id: id,
            name,
            version: Self::build_version(),
            net: RwLock::new(net),
            epoch,
            event_log,
            id_generator: ObjectIdGenerator::new(id, epoch),
            rpc_receiver: Mutex::new(rpc_receiver),
            history_log: RwLock::new(NetworkHistoryLog::new()),
            subscriber,
            policy_service,
        }
    }

    pub fn submit_event(&self, id: impl Into<ObjectId>, detail: impl Into<EventDetails>)
    {
        let id = id.into();
        let detail = detail.into();
        tracing::trace!("Submitting new event {:?} {:?}", id, detail);
        self.event_log.create_event(id, detail);
    }

    /// Retrieve the [`ObjectIdGenerator`] used to generate object identifiers
    pub fn ids(&self) -> &ObjectIdGenerator
    {
        &self.id_generator
    }

    /// Access the IRC network state
    pub fn network(&self) -> NetworkReadGuard
    {
        // XXX: This is read_recursive() and not read() because the current architecture
        // requires both `CommandProcessor` and the individual handlers to acquire the read
        // lock at the same time. If this were a normal read lock and the `Server` task
        // attempted to acquire the write lock in between those two points, it would deadlock.
        self.net.read_recursive()
    }

    /// Access the policy service
    pub fn policy(&self) -> &Policy
    {
        &self.policy_service
    }

    /// Access the network history
    pub fn history(&self) -> RwLockReadGuard<NetworkHistoryLog>
    {
        self.history_log.read()
    }

    /// Access the event log.
    pub fn event_log(&self) -> std::sync::RwLockReadGuard<EventLog>
    {
        self.event_log.event_log()
    }

    /// Access the replicated event log
    pub fn sync_log(&self) -> &ReplicatedEventLog
    {
        &self.event_log
    }

    /// Get the server's name
    pub fn name(&self) -> &ServerName
    {
        &self.name
    }

    /// The server's ID
    pub fn id(&self) -> ServerId
    {
        self.my_id
    }

    /// The server's epoch
    pub fn epoch(&self) -> EpochId
    {
        self.epoch
    }

    /// The server's build version
    pub fn version(&self) -> &str
    {
        &self.version
    }

    fn build_version() -> String
    {
        let git_version = crate::build_data::GIT_COMMIT_HASH.map(|s| format!("-{}", s)).unwrap_or_else(String::new);
        let git_dirty = if matches!(crate::build_data::GIT_DIRTY, Some(true)) {
            "-dirty".to_string()
        } else {
            String::new()
        };
        format!("sable-{}{}{}", crate::build_data::PKG_VERSION, git_version, git_dirty)
    }

    /// The server's build flags
    pub fn server_flags(&self) -> state::ServerFlags
    {
        let mut ret = state::ServerFlags::empty();
        if cfg!(feature = "debug")
        {
            ret |= state::ServerFlags::DEBUG;
        }
        ret
    }

    #[tracing::instrument(skip(self))]
    fn apply_event(&self, event: Event)
    {
        tracing::trace!("Applying inbound event");

        // We need to queue up the emitted updates and process them after `apply()` returns and we've released
        // the write lock on `net`. The handlers for various network updates require read access to `net`.
        let mut update_queue = crate::network::SavedUpdateReceiver::new();

        self.net.write().apply(&event, &update_queue).unwrap_or_else(|_| panic!("Event {:?} failed to apply", event));

        update_queue.playback(self);
    }

    #[tracing::instrument(skip_all)]
    pub async fn run(self: Arc<Self>,
                     mut shutdown_channel: oneshot::Receiver<ShutdownAction>
                ) -> ShutdownAction
    {
        self.submit_event(self.my_id, details::NewServer {
            epoch: self.epoch,
            name: self.name,
            ts: crate::utils::now(),
            flags: self.server_flags(),
            version: self.version().to_string(),
        });

        let mut check_ping_timer = time::interval(Duration::from_secs(60));

        let mut rpc_receiver = self.rpc_receiver.lock().await;

        let shutdown_action = loop
        {
            tracing::trace!("server run loop");

            select! {
                res = rpc_receiver.recv() =>
                {
                    tracing::trace!("...from rpc_receiver");
                    match res {
                        Some(NetworkMessage::NewEvent(event)) =>
                        {
                            self.apply_event(event);
                        },
                        Some(NetworkMessage::ImportNetworkState(new_net)) =>
                        {
                            tracing::debug!("Server got state import");
                            // Using replace() here because it works on a mut borrow of the destination;
                            // we can't assign directly to something held by RwLock
                            let _ = std::mem::replace(&mut *self.net.write(), *new_net);
                        },
                        Some(NetworkMessage::ExportNetworkState(channel)) =>
                        {
                            tracing::debug!("Server got state export request; sending");
                            let copied_net = {
                                self.net.read().clone()
                            };
                            channel.send(Box::new(copied_net)).await.or_log("Error sending network state for export");
                        },
                        None => {
                            panic!("what to do here?");
                        }
                    }
                },
                _ = check_ping_timer.tick() =>
                {
                    tracing::trace!("...from check_ping_timer");
                    self.check_pings();
                },
                shutdown = &mut shutdown_channel =>
                {
                    match shutdown
                    {
                        Err(e) =>
                        {
                            tracing::error!("Got error ({}) from shutdown channel; exiting", e);
                            break ShutdownAction::Shutdown;
                        }
                        Ok(ShutdownAction::Shutdown) | Ok(ShutdownAction::Restart) =>
                        {
                            // In either of these cases, we're disconnecting from the network and
                            // should announce that. We might be starting again, but it'll be from
                            // a clean slate.
                            break shutdown.unwrap();
                        }
                        Ok(ShutdownAction::Upgrade) =>
                        {
                            // If we're upgrading, then don't signal to the network that we're shutting down.
                            // The actual state save/restore will be called by main() after everything's stopped
                            // processing.
                            return ShutdownAction::Upgrade;
                        }
                    }
                },
            }
        };

        let net = self.net.read();
        let me = net.server(self.my_id).expect("Couldn't say I quit as I have no record of myself");

        self.submit_event(self.my_id, details::ServerQuit { epoch: me.epoch() });

        shutdown_action
    }
}

