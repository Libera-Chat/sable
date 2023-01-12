use super::*;
use AuthenticateStatus::*;

/// Result type for SASL authentication methods. An Ok value represents data to be sent
/// to the client to continue authentication; an Err value signals that the login attempt
/// was unsuccessful for some reason.
pub type SaslResult = Result<AuthenticateStatus, CommandError>;

pub trait SaslMechanism<DB> : Send + Sync + 'static
{
    fn step(&self, server: &ServicesServer<DB>, session: &SaslSession, data: Vec<u8>) -> SaslResult;
}

pub fn build_mechanisms<DB: DatabaseConnection>() -> HashMap<String, Box<dyn SaslMechanism<DB>>>
{
    let mut ret = HashMap::<String, Box<dyn SaslMechanism<DB>>>::new();

    ret.insert("PLAIN".to_owned(), Box::new(plain::SaslPlain));

    ret
}

mod plain;