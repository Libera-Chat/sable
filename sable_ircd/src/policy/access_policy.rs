use super::*;

/// An `AccessPolicyService` is used to make decisions about access for clients
/// to the network.
#[delegatable_trait]
pub trait AccessPolicyService
{
    /// Decide whether the given [`ClientConnection`] is permitted to connect.
    ///
    /// This is called after nickname and username information is collected; these
    /// will be accessible via `client.pre_client`.
    fn check_user_access(&self, server: &crate::ClientServer, net: &Network, client: &ClientConnection) -> bool;
}
