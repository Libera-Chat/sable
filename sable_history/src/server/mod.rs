use std::convert::Infallible;
use std::sync::Arc;

use anyhow::{Context, Result};
use diesel::migration::MigrationSource;
use diesel::prelude::*;
use diesel_async::async_connection_wrapper::AsyncConnectionWrapper;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use itertools::Itertools;
use serde::Deserialize;
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};
use tracing::instrument;

use sable_network::prelude::*;
use sable_server::ServerType;

mod sync;
mod update_handler;

/// Advisory lock key for serializing database migrations across concurrent processes.
const MIGRATION_LOCK_KEY: i64 = 0x5361626c48697374; // value is "SablHist"

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[derive(Debug, Clone, Deserialize)]
pub struct HistoryServerConfig {
    pub database: String,
    pub auto_run_migrations: bool,
}

pub struct HistoryServer {
    node: Arc<NetworkNode>,
    history_receiver: Mutex<UnboundedReceiver<sable_network::rpc::NetworkHistoryUpdate>>,
    database_connection: Mutex<AsyncPgConnection>, // TODO: use a connection pool
}

impl ServerType for HistoryServer {
    type Config = HistoryServerConfig;

    type ProcessedConfig = HistoryServerConfig;

    type ConfigError = Infallible;

    type Saved = ();

    fn validate_config(config: &Self::Config) -> Result<Self::ProcessedConfig, Self::ConfigError> {
        Ok(config.clone())
    }

    async fn new(
        config: Self::ProcessedConfig,
        _tls_data: &sable_network::config::TlsData,
        node: std::sync::Arc<sable_network::prelude::NetworkNode>,
        history_receiver: tokio::sync::mpsc::UnboundedReceiver<
            sable_network::rpc::NetworkHistoryUpdate,
        >,
    ) -> anyhow::Result<Self> {
        let database = config.database.clone();
        if config.auto_run_migrations {
            tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
                // run_pending_migrations only support sync connections
                let mut conn = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&database)
                    .context("Couldn't connect to database")?;

                // Prevent multiple migrations from running at the same time, or processes starting
                // while migrations are still running.
                diesel::sql_query(format!("SELECT pg_advisory_lock({MIGRATION_LOCK_KEY})"))
                    .execute(&mut conn)
                    .context("Couldn't acquire migration advisory lock")?;

                let res = (|| -> Result<_> {
                    tracing::info!("Running database migrations");
                    tracing::trace!(
                        "Required migrations: {}",
                        MIGRATIONS
                            .migrations()
                            .map_err(|e| anyhow::anyhow!("Couldn't get migrations: {e}"))?
                            .iter()
                            .map(diesel::migration::Migration::<diesel::pg::Pg>::name)
                            .join(", ")
                    );
                    let migrations = conn
                        .run_pending_migrations(MIGRATIONS)
                        .map_err(|e| anyhow::anyhow!("Database migrations failed: {e}"))?;
                    if migrations.is_empty() {
                        tracing::info!("No database migrations to run");
                    } else {
                        tracing::info!(
                            "Applied database migrations: {}",
                            migrations.iter().map(ToString::to_string).join(", ")
                        )
                    }
                    Ok(())
                })();

                diesel::sql_query(format!("SELECT pg_advisory_unlock({MIGRATION_LOCK_KEY})"))
                    .execute(&mut conn)
                    .context("Couldn't release migration advisory lock")?;

                res
            })
            .await
            .context("Couldn't join migration task")??;
        }
        Ok(Self {
            node,
            history_receiver: Mutex::new(history_receiver),
            database_connection: Mutex::new(
                AsyncPgConnection::establish(&config.database)
                    .await
                    .context("Couldn't connect to database")?,
            ),
        })
    }

    async fn run(
        self: std::sync::Arc<Self>,
        mut shutdown_channel: tokio::sync::broadcast::Receiver<sable_network::rpc::ShutdownAction>,
    ) {
        let mut history_receiver = self.history_receiver.lock().await;

        loop {
            tokio::select! {
                _ = shutdown_channel.recv() => { break; }

                update = history_receiver.recv() =>
                {
                    let Some(update) = update else { break; };

                    if let NetworkStateChange::NewServer(new_server) = &update.change
                    {
                        if new_server.server == self.node.id()
                        {
                            self.burst_to_network().await;
                        }
                    }

                    if let Err(error) = self.handle_history_update(update).await {
                        tracing::error!(?error, "Error return handling history update");
                    }
                }
            }
        }
    }

    async fn shutdown(self) {}

    async fn save(self) -> Result<Self::Saved, sable_server::ServerSaveError> {
        Ok(())
    }

    fn restore(
        _state: Self::Saved,
        _node: std::sync::Arc<sable_network::prelude::NetworkNode>,
        _history_receiver: tokio::sync::mpsc::UnboundedReceiver<
            sable_network::rpc::NetworkHistoryUpdate,
        >,
        _config: &Self::ProcessedConfig,
    ) -> std::io::Result<Self> {
        unimplemented!("history servers can't hot-upgrade");
    }

    #[instrument(skip_all)]
    async fn handle_remote_command(
        &self,
        req: sable_network::rpc::RemoteServerRequestType,
    ) -> sable_network::rpc::RemoteServerResponse {
        tracing::debug!(?req, "Got remote request");

        use crate::server::rpc::RemoteServerRequestType::*;
        use sable_network::rpc::RemoteServerResponse;

        match req {
            History(req) => {
                use crate::server::rpc::RemoteHistoryServerRequestType::*;
                use crate::server::rpc::RemoteHistoryServerResponse::*;

                let history_service = crate::PgHistoryService::new(&self.database_connection);
                match req {
                    ListTargets {
                        user,
                        after_ts,
                        before_ts,
                        limit,
                    } => TargetList(
                        history_service
                            .list_targets(user, after_ts, before_ts, limit)
                            .await
                            .into_iter()
                            .collect(),
                    ),
                    GetEntries {
                        user,
                        target,
                        request,
                    } => Entries(
                        history_service
                            .get_entries(user, target, request)
                            .await
                            .map(|entries| entries.into_iter().collect()),
                    ),
                }
                .into()
            }
            Services(_) => {
                tracing::warn!(?req, "Got unsupported request (services)");
                RemoteServerResponse::NotSupported
            }
            Ping => {
                tracing::warn!(?req, "Got unsupported request (ping)");
                RemoteServerResponse::NotSupported
            }
        }
    }
}
