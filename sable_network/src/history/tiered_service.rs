use std::collections::hash_map::{Entry, HashMap};

use tracing::instrument;

use crate::prelude::*;

/// Implementation of [`HistoryService`] backed by two other services, one fast and short-lived and
/// the other slower and longer-lived.
///
/// This is used to query [`HistoryService`] when possible, and fall-back to a remote server when
/// events expired locally.
pub struct TieredHistoryService<
    FastService: HistoryService + Send + Sync,
    SlowService: HistoryService + Send + Sync,
> {
    fast_service: Option<FastService>,
    slow_service: Option<SlowService>,
}

impl<FastService: HistoryService + Send + Sync, SlowService: HistoryService + Send + Sync>
    TieredHistoryService<FastService, SlowService>
{
    pub fn new(fast_service: Option<FastService>, slow_service: Option<SlowService>) -> Self {
        Self {
            fast_service,
            slow_service,
        }
    }
}

impl<FastService: HistoryService + Send + Sync, SlowService: HistoryService + Send + Sync>
    HistoryService for TieredHistoryService<FastService, SlowService>
{
    #[instrument(skip(self))]
    async fn list_targets(
        &self,
        user: UserId,
        after_ts: Option<i64>,
        before_ts: Option<i64>,
        limit: Option<usize>,
    ) -> HashMap<TargetId, i64> {
        match (&self.fast_service, &self.slow_service) {
            (Some(fast_service), Some(slow_service)) => {
                let (mut targets1, mut targets2) = futures::join!(
                    slow_service.list_targets(user, after_ts, before_ts, limit),
                    fast_service.list_targets(user, after_ts, before_ts, limit)
                );

                // merge targets, taking the most recent timestamp for those present
                // in both backends
                if targets1.len() < targets2.len() {
                    (targets1, targets2) = (targets2, targets1);
                }
                for (target, ts) in targets2.drain() {
                    match targets1.entry(target) {
                        Entry::Occupied(mut entry) => {
                            if *entry.get() < ts {
                                entry.insert(ts);
                            }
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(ts);
                        }
                    }
                }
                targets1
            }
            (None, Some(slow_service)) => {
                slow_service
                    .list_targets(user, after_ts, before_ts, limit)
                    .await
            }
            (Some(fast_service), None) => {
                fast_service
                    .list_targets(user, after_ts, before_ts, limit)
                    .await
            }
            (None, None) => HashMap::new(),
        }
    }

    #[instrument(skip(self))]
    async fn get_entries(
        &self,
        user: UserId,
        target: TargetId,
        request: HistoryRequest,
    ) -> Result<impl IntoIterator<Item = HistoricalEvent>, HistoryError> {
        // It's tempting to return Box<dyn IntoIterator> here instead of collecting into a
        // temporary Vec, but we can't because IntoIterator::IntoIter potentially differs
        match (&self.fast_service, &self.slow_service) {
            (_, Some(slow_service)) => {
                // TODO: implement fallback
                tracing::info!("get_entries slow");
                let entries = slow_service.get_entries(user, target, request).await?;
                Ok(entries.into_iter().collect())
            }
            (Some(fast_service), None) => {
                tracing::info!("get_entries fast");
                let entries = fast_service.get_entries(user, target, request).await?;
                Ok(entries.into_iter().collect())
            }
            (None, None) => Ok(Vec::new()),
        }
    }
}
