use std::str::FromStr;

use super::*;
use sable_network::prelude::*;

pub struct SaslPlain;

impl<DB: DatabaseConnection> SaslMechanism<DB> for SaslPlain {
    fn step(
        &self,
        server: &ServicesServer<DB>,
        _session: &SaslSession,
        data: Vec<u8>,
    ) -> SaslResult {
        let elements = data.split(|e| *e == 0).collect::<Vec<_>>();

        let (account_name, password) = match elements.as_slice() {
            // Derive authzid if not provided
            [&[], authcid, passwd] => (authcid, passwd),
            // PLAIN specifies both authzid and authcid; we don't support those two being different
            [authzid, authcid, passwd] if authzid == authcid => (authcid, passwd),
            _ => return Ok(Fail),
        };

        let account_name = std::str::from_utf8(account_name)?;
        let account_name = Nickname::from_str(account_name)?;
        let account = server.db.account_named(&account_name)?;

        let auth = server.db.auth_for_account(account.id)?;

        match bcrypt::verify(password, &auth.password_hash) {
            Ok(true) => {
                tracing::debug!(?account_name, "sasl login successful");
                Ok(Success(account.id))
            }
            Ok(false) => {
                tracing::debug!("wrong password");
                Ok(Fail)
            }
            Err(_) => Err("Couldn't verify password".into()),
        }
    }
}
