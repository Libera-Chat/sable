use sable_network::prelude::ban::UserDetails;

use super::*;

/// An error type describing reasons why a client may be denied access
#[derive(Debug, Clone)]
pub enum AccessError {
    /// User matched a network ban, with provided reason
    Banned(String),
}

impl ClientServer {
    pub(super) fn check_user_access(
        &self,
        net: &Network,
        client: &ClientConnection,
    ) -> Result<(), AccessError> {
        if let Some(pre_client) = client.pre_client() {
            let ip = client.remote_addr();
            let user_details = UserDetails {
                nick: pre_client.nick.get().map(Nickname::as_ref),
                ident: pre_client.user.get().map(Username::as_ref),
                host: pre_client.hostname.get().map(Hostname::as_ref),
                ip: Some(&ip),
                realname: pre_client.realname.get().map(String::as_ref),
            };

            if let Some(ban) = net.network_bans().find(&user_details) {
                return Err(AccessError::Banned(ban.reason.clone()));
            }
        }
        Ok(())
    }
}
