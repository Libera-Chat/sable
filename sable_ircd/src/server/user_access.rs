use sable_network::prelude::ban::*;

use super::*;

/// An error type describing reasons why a client may be denied access
#[derive(Debug, Clone)]
pub enum AccessError {
    /// User matched a network ban, with provided reason
    Banned(String),
    /// An internal error occurred while attempting to verify access
    InternalError,
}

impl ClientServer {
    #[tracing::instrument(skip(self, net))]
    pub(super) fn check_user_access(
        &self,
        net: &Network,
        client: &ClientConnection,
    ) -> Result<(), AccessError> {
        if let Some(pre_client) = client.pre_client() {
            let Some(nick) = pre_client.nick.get().cloned() else {
                tracing::error!("PreClient nickname not set");
                return Err(AccessError::InternalError);
            };
            let Some(user) = pre_client.user.get().cloned() else {
                tracing::error!("PreClient username not set");
                return Err(AccessError::InternalError);
            };
            let Some(host) = pre_client.hostname.get().cloned() else {
                tracing::error!("PreClient hostname not set");
                return Err(AccessError::InternalError);
            };
            let Some(realname) = pre_client.realname.get().cloned() else {
                tracing::error!("PreClient realname not set");
                return Err(AccessError::InternalError);
            };
            let Some((user_param_1, user_param_2)) = pre_client.extra_user_params.get().cloned()
            else {
                tracing::error!("PreClient user parameters not set");
                return Err(AccessError::InternalError);
            };

            let ip = client.remote_addr();
            let tls = client.connection.is_tls();

            let user_details = PreRegistrationBanSettings {
                nick,
                user,
                host,
                realname,
                ip,
                user_param_1,
                user_param_2,
                tls,
            };

            if let Some(ban) = net.network_bans().find(&user_details) {
                return Err(AccessError::Banned(ban.reason.clone()));
            }
        }
        Ok(())
    }
}
