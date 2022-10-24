use super::*;

/// Saved state of a [`NetworkNode`] for later resumption
#[derive(serde::Serialize,serde::Deserialize)]
pub struct NetworkNodeState<Policy = crate::policy::StandardPolicyService>
    where Policy: PolicyService + Saveable
{
    id: ServerId,
    name: ServerName,
    net: Network,
    epoch: EpochId,
    id_generator: ObjectIdGenerator,
    history_log: NetworkHistoryLog,
    policy_state: Policy::Saved,
}

impl<Policy: PolicyService + Saveable> NetworkNode<Policy>
{
    /// Save the node's state for later resumption
    pub fn save_state(self) -> NetworkNodeState<Policy>
    {
        NetworkNodeState {
            id: self.my_id,
            name: self.name,
            net: Arc::try_unwrap(self.net.into_inner()).unwrap(),
            epoch: self.epoch,
            id_generator: self.id_generator,
            history_log: self.history_log.into_inner(),
            policy_state: self.policy_service.save(),
        }
    }

    /// Restore from a previously saved state.
    ///
    /// The `listener_collection` is only used during the resumption to restore
    /// connection data; the other arguments are as for [`new`](Self::new).
    pub fn restore_from(
            state: NetworkNodeState<Policy>,
            event_log: Arc<ReplicatedEventLog>,
            rpc_receiver: UnboundedReceiver<NetworkMessage>,
            subscriber: UnboundedSender<NetworkHistoryUpdate>,
        ) -> std::io::Result<Self>
    {
        Ok(Self {
            my_id: state.id,
            name: state.name,
            version: Self::build_version(),
            net: RwLock::new(Arc::new(state.net)),
            epoch: state.epoch,
            id_generator: state.id_generator,
            event_log,
            rpc_receiver: Mutex::new(rpc_receiver),
            history_log: RwLock::new(state.history_log),
            subscriber,
            policy_service: Policy::restore(state.policy_state),
        })
    }
}