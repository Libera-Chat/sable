use super::*;
use AuthenticateStatus::*;
use RemoteServicesServerResponse::Authenticate;

impl<DB: DatabaseConnection> ServicesServer<DB> {
    pub fn begin_authenticate(&self, session: SaslSessionId, mechanism: String) -> CommandResult {
        if self.sasl_sessions.contains_key(&session) {
            return Ok(Authenticate(Fail).into());
        }

        if !self.sasl_mechanisms.contains_key(&mechanism) {
            return Ok(Authenticate(Fail).into());
        }

        self.sasl_sessions.insert(
            session,
            SaslSession {
                id: session,
                mechanism,
            },
        );
        Ok(Authenticate(InProgress(Vec::new())).into())
    }

    pub fn authenticate(&self, session_id: SaslSessionId, data: Vec<u8>) -> CommandResult {
        let session_entry = self.sasl_sessions.entry(session_id);
        let dashmap::mapref::entry::Entry::Occupied(session_entry) = session_entry else {
            return Ok(Authenticate(Fail).into());
        };
        let session = session_entry.get();

        let Some(mechanism) = self.sasl_mechanisms.get(&session.mechanism) else {
            session_entry.remove();
            return Ok(Authenticate(Fail).into());
        };

        match mechanism.step(self, session, data) {
            Ok(response) => Ok(Authenticate(response).into()),
            Err(e) => {
                tracing::debug!(?session_id, "SASL {} step failed: {e}", mechanism.name());
                // Equivalent to self.fail_authenticate(session_id) but we can't call it here
                // because we already have the lock.
                session_entry.remove();
                Ok(Authenticate(Fail).into())
            }
        }
    }

    pub fn abort_authenticate(&self, session_id: SaslSessionId) -> CommandResult {
        self.sasl_sessions.remove(&session_id);
        Ok(Authenticate(Aborted).into())
    }

    pub fn fail_authenticate(&self, session_id: SaslSessionId) -> CommandResult {
        self.sasl_sessions.remove(&session_id);
        Ok(Authenticate(Fail).into())
    }
}
