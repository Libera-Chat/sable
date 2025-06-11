use std::collections::HashMap;
use std::num::NonZeroUsize;

use tracing::instrument;

use crate::prelude::*;
use crate::rpc::*;

/// Implementation of [`HistoryService`] that forwards requests to a `HistoryServer`
/// through the RPC.
pub struct RemoteHistoryService<'a, NetworkPolicy: policy::PolicyService> {
    node: &'a NetworkNode<NetworkPolicy>,
    remote_server_name: ServerName,
}

impl<'a, NetworkPolicy: policy::PolicyService> RemoteHistoryService<'a, NetworkPolicy> {
    pub fn new(node: &'a NetworkNode<NetworkPolicy>, remote_server_name: ServerName) -> Self {
        RemoteHistoryService {
            node,
            remote_server_name,
        }
    }
}

impl<NetworkPolicy: policy::PolicyService> HistoryService
    for RemoteHistoryService<'_, NetworkPolicy>
{
    #[instrument(skip(self))]
    async fn list_targets(
        &self,
        user: UserId,
        after_ts: Option<i64>,
        before_ts: Option<i64>,
        limit: Option<NonZeroUsize>,
    ) -> HashMap<TargetId, i64> {
        let res = self
            .node
            .sync_log()
            .send_remote_request(
                self.remote_server_name,
                RemoteHistoryServerRequestType::ListTargets {
                    user,
                    after_ts,
                    before_ts,
                    limit,
                }
                .into(),
            )
            .await;
        tracing::trace!("list_targets RPC response: {res:?}");
        match res {
            Ok(RemoteServerResponse::History(RemoteHistoryServerResponse::TargetList(
                targets,
            ))) => targets.into_iter().collect(),
            Ok(RemoteServerResponse::History(_))
            | Ok(RemoteServerResponse::Services(_))
            // This request should never error
            | Ok(RemoteServerResponse::Error(_))
            | Ok(RemoteServerResponse::NotSupported)
            | Ok(RemoteServerResponse::Success) => {
                tracing::error!("Got unexpected response to ListTargets request: {res:?}");
                HashMap::new()
            },
            Err(e) => {
                tracing::error!("ListTargets request failed: {e:?}");
                HashMap::new()
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_entries(
        &self,
        user: UserId,
        target: TargetId,
        request: HistoryRequest,
    ) -> Result<impl IntoIterator<Item = HistoricalEvent>, HistoryError> {
        let res = self
            .node
            .sync_log()
            .send_remote_request(
                self.remote_server_name,
                rpc::RemoteHistoryServerRequestType::GetEntries {
                    user,
                    target,
                    request,
                }
                .into(),
            )
            .await;
        match res {
            Ok(RemoteServerResponse::History(RemoteHistoryServerResponse::Entries(
                entries,
            ))) => {
                tracing::trace!("get_entries RPC response: {}", entries.is_ok());
                entries
            },
            Ok(RemoteServerResponse::History(_))
            | Ok(RemoteServerResponse::Services(_))
            // Errors while processing this request would return Entries(Err(_))
            | Ok(RemoteServerResponse::Error(_))
            | Ok(RemoteServerResponse::NotSupported)
            | Ok(RemoteServerResponse::Success) => {
                tracing::error!("Got unexpected response to GetEntries request: {res:?}");
                Ok(Vec::new())
            },
            Err(e) => {
                tracing::error!("GetEntries request failed: {e:?}");
                Ok(Vec::new())
            }
        }
    }
}
