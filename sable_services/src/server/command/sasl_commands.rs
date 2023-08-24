use super::*;
use AuthenticateStatus::*;
use RemoteServerResponse::Authenticate;

impl<DB: DatabaseConnection> ServicesServer<DB> {
    pub fn begin_authenticate(&self, session: SaslSessionId, mechanism: String) -> CommandResult {
        if self.sasl_sessions.contains_key(&session) {
            return Ok(Authenticate(Aborted));
        }

        if !self.sasl_mechanisms.contains_key(&mechanism) {
            return Ok(Authenticate(Aborted));
        }

        self.sasl_sessions.insert(
            session,
            SaslSession {
                id: session,
                mechanism,
            },
        );
        Ok(Authenticate(InProgress(Vec::new())))
    }

    pub fn authenticate(&self, session_id: SaslSessionId, data: Vec<u8>) -> CommandResult {
        let Some(session) = self.sasl_sessions.get(&session_id) else {
            return Ok(Authenticate(Aborted));
        };

        let Some(mechanism) = self.sasl_mechanisms.get(&session.mechanism) else {
            self.sasl_sessions.remove(&session_id);
            return Ok(Authenticate(Aborted));
        };

        let response = mechanism.step(self, &session, data)?;

        Ok(Authenticate(response))
    }

    pub fn abort_authenticate(&self, session_id: SaslSessionId) -> CommandResult {
        self.sasl_sessions.remove(&session_id);
        Ok(Authenticate(Aborted))
    }
}
