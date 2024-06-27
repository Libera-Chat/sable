//! Implementation of [IRCv3 MONITOR](https://ircv3.net/specs/extensions/monitor)
//!
//! Monitors are connection-specific (not user-wide), and not propagated across the network.
//! Therefore, they are identified only by a `ConnectionId`.

use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Context, Result};
use thiserror::Error;

use crate::make_numeric;
use crate::messages::MessageSink;
use crate::prelude::*;
use crate::ClientServer;
use client_listener::ConnectionId;
use sable_network::prelude::*;
use sable_network::validated::Nickname;

#[derive(Error, Clone, Debug)]
pub enum MonitorInsertError {
    #[error("this connection has too many monitors ({current}), maximum is {max}")]
    /// `current` may be greater than `max` if server configuration was edited.
    TooManyMonitorsPerConnection { max: usize, current: usize },
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct MonitorSet {
    pub max_per_connection: usize,
    monitors_by_connection: HashMap<ConnectionId, HashSet<Nickname>>,
    monitors_by_nickname: HashMap<Nickname, HashSet<ConnectionId>>,
}

impl MonitorSet {
    pub fn new(max_per_connection: usize) -> MonitorSet {
        MonitorSet {
            max_per_connection,
            monitors_by_connection: HashMap::new(),
            monitors_by_nickname: HashMap::new(),
        }
    }

    /// Marks the `nick` as being monitored by the given connection
    pub fn insert(
        &mut self,
        nick: Nickname,
        monitor: ConnectionId,
    ) -> Result<(), MonitorInsertError> {
        let entry = self.monitors_by_connection.entry(monitor).or_default();
        if entry.len() >= self.max_per_connection {
            return Err(MonitorInsertError::TooManyMonitorsPerConnection {
                max: self.max_per_connection,
                current: entry.len(),
            });
        }
        entry.insert(nick);
        self.monitors_by_nickname
            .entry(nick)
            .or_default()
            .insert(monitor);
        Ok(())
    }

    /// Marks the `nick` as no longer monitored by the given connection
    ///
    /// Returns whether the nick was indeed monitored by the connection.
    pub fn remove(&mut self, nick: Nickname, monitor: ConnectionId) -> bool {
        self.monitors_by_connection
            .get_mut(&monitor)
            .map(|set| set.remove(&nick));
        self.monitors_by_nickname
            .get_mut(&nick)
            .map(|set| set.remove(&monitor))
            .unwrap_or(false)
    }

    /// Remove all monitors of a connection
    ///
    /// Returns the set of nicks the connection monitored, if any.
    pub fn remove_connection(&mut self, monitor: ConnectionId) -> Option<HashSet<Nickname>> {
        let nicks = self.monitors_by_connection.remove(&monitor);
        if let Some(nicks) = &nicks {
            for nick in nicks {
                self.monitors_by_nickname
                    .get_mut(nick)
                    .expect("monitors_by_nickname missing nick present in monitors_by_connection")
                    .remove(&monitor);
            }
        }
        nicks
    }

    /// Returns all connections monitoring the given nick
    pub fn nick_monitors(&self, nick: &Nickname) -> Option<&HashSet<ConnectionId>> {
        self.monitors_by_nickname.get(nick)
    }

    /// Returns all nicks monitored by the given connection
    pub fn monitored_nicks(&self, monitor: ConnectionId) -> Option<&HashSet<Nickname>> {
        self.monitors_by_connection.get(&monitor)
    }
}

/// Trait of [`NetworkStateChange`] details that are relevant to connections using
/// [IRCv3 MONITOR](https://ircv3.net/specs/extensions/monitor) to monitor users.
pub(crate) trait MonitoredItem: std::fmt::Debug {
    /// Same as [`try_notify_monitors`] but logs errors instead of returning `Result`.
    fn notify_monitors(&self, server: &ClientServer) {
        if let Err(e) = self.try_notify_monitors(server) {
            tracing::error!("Error while notifying monitors of {:?}: {}", self, e);
        }
    }

    /// Send `RPL_MONONLINE`/`RPL_MONOFFLINE` to all connections monitoring nicks involved in this
    /// event
    fn try_notify_monitors(&self, server: &ClientServer) -> Result<()>;
}

impl MonitoredItem for update::NewUser {
    fn try_notify_monitors(&self, server: &ClientServer) -> Result<()> {
        notify_monitors(server, &self.user.nickname, || {
            make_numeric!(MonOnline, &self.user.nuh())
        })
    }
}

impl MonitoredItem for update::UserNickChange {
    fn try_notify_monitors(&self, server: &ClientServer) -> Result<()> {
        if self.user.nickname != self.new_nick {
            // Don't notify on case change
            notify_monitors(server, &self.user.nickname, || {
                make_numeric!(MonOffline, self.user.nickname.as_ref())
            })?;
            notify_monitors(server, &self.new_nick, || {
                make_numeric!(
                    MonOnline,
                    &state::HistoricUser {
                        nickname: self.new_nick,
                        ..self.user.clone()
                    }
                    .nuh()
                )
            })?;
        }
        Ok(())
    }
}

impl MonitoredItem for update::UserQuit {
    fn try_notify_monitors(&self, server: &ClientServer) -> Result<()> {
        notify_monitors(server, &self.user.nickname, || {
            make_numeric!(MonOffline, self.user.nickname.as_ref())
        })
    }
}

impl MonitoredItem for update::BulkUserQuit {
    fn try_notify_monitors(&self, server: &ClientServer) -> Result<()> {
        self.items
            .iter()
            .map(|item| item.try_notify_monitors(server))
            .collect::<Vec<_>>() // Notify all monitors even if one of them fails halfway
            .into_iter()
            .collect()
    }
}

fn notify_monitors(
    server: &ClientServer,
    nick: &Nickname,
    mut make_numeric: impl FnMut() -> UntargetedNumeric,
) -> Result<()> {
    // Copying the set of monitors to release lock on `server.monitors` ASAP
    let monitors: Option<Vec<_>> = server
        .monitors
        .read()
        .monitors_by_nickname
        .get(nick)
        .map(|monitors| monitors.iter().copied().collect());
    if let Some(monitors) = monitors {
        let network = server.network();
        monitors
            .into_iter()
            .map(|monitor| -> Result<()> {
                let Some(conn) = server.find_connection(monitor) else {
                    // TODO: Remove from monitors?
                    return Ok(());
                };
                let user_id = conn
                    .user_id()
                    .ok_or(anyhow!("Monitor by user with no user_id {:?}", conn.id()))?;
                let monitor_user = network
                    .user(user_id)
                    .context("Could not find monitoring user")?;
                conn.send(make_numeric().format_for(server, &monitor_user));
                Ok(())
            })
            .collect::<Vec<_>>() // Notify all monitors even if one of them fails halfway
            .into_iter()
            .collect()
    } else {
        Ok(())
    }
}
