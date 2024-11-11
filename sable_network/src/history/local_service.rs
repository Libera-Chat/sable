use std::collections::HashMap;

use tracing::instrument;

use crate::network::state::HistoricMessageTargetId;
use crate::prelude::*;

/// Helper to extract the target name for chathistory purposes from a given event.
///
/// This might be the source or target of the actual event, or might be None if it's
/// an event type that we don't include in history playback
fn target_id_for_entry(for_user: UserId, entry: &HistoryLogEntry) -> Option<TargetId> {
    match &entry.details {
        NetworkStateChange::NewMessage(message) => match &message.target {
            HistoricMessageTargetId::User(user) if user.user() == &for_user => {
                (&message.source).try_into().ok()
            }
            _ => (&message.target).try_into().ok(),
        },
        _ => None,
    }
}

/// Implementation of [`HistoryService`] backed by [`NetworkNode`]
pub struct LocalHistoryService<'a, NetworkPolicy: policy::PolicyService> {
    node: &'a NetworkNode<NetworkPolicy>,
}

impl<'a, NetworkPolicy: policy::PolicyService> LocalHistoryService<'a, NetworkPolicy> {
    pub fn new(node: &'a NetworkNode<NetworkPolicy>) -> Self {
        LocalHistoryService { node }
    }

    fn get_history_for_target(
        &self,
        source: UserId,
        target: TargetId,
        from_ts: Option<i64>,
        to_ts: Option<i64>,
        backward_limit: usize,
        forward_limit: usize,
    ) -> Result<impl Iterator<Item = HistoricalEvent>, HistoryError> {
        let mut backward_entries = Vec::new();
        let mut forward_entries = Vec::new();
        let mut target_exists = false;

        // Keep the lock on the NetworkHistoryLog between the backward and the forward
        // search to make sure both have a consistent state
        let log = self.node.history();
        let net = self.node.network();

        if backward_limit != 0 {
            let from_ts = if forward_limit == 0 {
                from_ts
            } else {
                // HACK: This is AROUND so we want to capture messages whose timestamp matches exactly
                // (it's a message in the middle of the range)
                from_ts.map(|from_ts| from_ts + 1)
            };

            for entry in log.entries_for_user_reverse(source) {
                target_exists = true;
                if matches!(from_ts, Some(ts) if entry.timestamp >= ts) {
                    // Skip over until we hit the timestamp window we're interested in
                    continue;
                }
                if matches!(to_ts, Some(ts) if entry.timestamp <= ts) {
                    // If we hit this then we've passed the requested window and should stop
                    break;
                }

                if let Some(event_target) = target_id_for_entry(source, entry) {
                    if event_target == target {
                        backward_entries.push(entry.clone());
                    }
                }

                if backward_limit <= backward_entries.len() {
                    break;
                }
            }
        }

        if forward_limit != 0 {
            for entry in log.entries_for_user(source) {
                target_exists = true;
                if matches!(from_ts, Some(ts) if entry.timestamp <= ts) {
                    // Skip over until we hit the timestamp window we're interested in
                    continue;
                }
                if matches!(to_ts, Some(ts) if entry.timestamp >= ts) {
                    // If we hit this then we've passed the requested window and should stop
                    break;
                }

                if let Some(event_target) = target_id_for_entry(source, entry) {
                    if event_target == target {
                        forward_entries.push(entry.clone());
                    }
                }

                if forward_limit <= forward_entries.len() {
                    break;
                }
            }
        }

        if target_exists {
            // "The order of returned messages within the batch is implementation-defined, but SHOULD be
            // ascending time order or some approximation thereof, regardless of the subcommand used."
            // -- https://ircv3.net/specs/extensions/chathistory#returned-message-notes
            Ok(backward_entries
                .into_iter()
                .rev()
                .chain(forward_entries)
                .flat_map(move |entry| Self::translate_log_entry(entry, &net)))
        } else {
            Err(HistoryError::InvalidTarget(target))
        }
    }

    fn translate_log_entry(entry: HistoryLogEntry, net: &Network) -> Option<HistoricalEvent> {
        match entry.details {
            NetworkStateChange::NewMessage(update::NewMessage {
                message,
                source: _,
                target: _,
            }) => {
                let message = net.message(message).ok()?;
                let source = message.source().ok()?;
                let target = message.target().ok()?;

                Some(HistoricalEvent::Message {
                    id: message.id(),
                    timestamp: entry.timestamp(), // update's timestamp, may differ from the message's timestamp
                    message_type: message.message_type(),
                    source: source.nuh(),
                    source_account: source.account_name().map(|n| n.to_string()),
                    target: target.to_string(),
                    text: message.text().to_string(),
                })
            }
            _ => None,
        }
    }
}

impl<NetworkPolicy: policy::PolicyService> HistoryService
    for LocalHistoryService<'_, NetworkPolicy>
{
    #[instrument(skip(self))]
    async fn list_targets(
        &self,
        user: UserId,
        after_ts: Option<i64>,
        before_ts: Option<i64>,
        limit: Option<usize>,
    ) -> HashMap<TargetId, i64> {
        let mut found_targets = HashMap::new();

        for entry in self.node.history().entries_for_user_reverse(user) {
            if matches!(after_ts, Some(ts) if entry.timestamp >= ts) {
                // Skip over until we hit the timestamp window we're interested in
                continue;
            }
            if matches!(before_ts, Some(ts) if entry.timestamp <= ts) {
                // We're iterating backwards through time; if we hit this then we've
                // passed the requested window and should stop
                break;
            }

            if let Some(target_id) = target_id_for_entry(user, entry) {
                found_targets.entry(target_id).or_insert(entry.timestamp);
            }

            // If this pushes us past the the requested limit, stop
            if matches!(limit, Some(limit) if limit <= found_targets.len()) {
                break;
            }
        }

        tracing::trace!("list_targets local response: {found_targets:?}");

        found_targets
    }

    #[instrument(skip(self))]
    async fn get_entries(
        &self,
        user: UserId,
        target: TargetId,
        request: HistoryRequest,
    ) -> Result<impl IntoIterator<Item = HistoricalEvent>, HistoryError> {
        let res = match request {
            #[rustfmt::skip]
            HistoryRequest::Latest { to_ts, limit } => self.get_history_for_target(
                user,
                target,
                None,
                to_ts,
                limit,
                0, // Forward limit
            ),

            HistoryRequest::Before { from_ts, limit } => {
                self.get_history_for_target(
                    user,
                    target,
                    Some(from_ts),
                    None,
                    limit,
                    0, // Forward limit
                )
            }
            HistoryRequest::After { start_ts, limit } => self.get_history_for_target(
                user,
                target,
                Some(start_ts),
                None,
                0, // Backward limit
                limit,
            ),
            HistoryRequest::Around { around_ts, limit } => {
                self.get_history_for_target(
                    user,
                    target,
                    Some(around_ts),
                    None,
                    limit / 2, // Backward limit
                    limit / 2, // Forward limit
                )
            }
            HistoryRequest::Between {
                start_ts,
                end_ts,
                limit,
            } => {
                if start_ts <= end_ts {
                    self.get_history_for_target(
                        user,
                        target,
                        Some(start_ts),
                        Some(end_ts),
                        0, // Backward limit
                        limit,
                    )
                } else {
                    // Search backward from start_ts instead of swapping start_ts and end_ts,
                    // because we want to match the last messages first in case we reach the limit
                    self.get_history_for_target(
                        user,
                        target,
                        Some(start_ts),
                        Some(end_ts),
                        limit,
                        0, // Forward limit
                    )
                }
            }
        };
        tracing::trace!("get_entries local response: {}", res.is_ok());
        res
    }
}
