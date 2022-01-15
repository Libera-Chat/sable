use super::*;

#[delegatable_trait]
pub trait AccessPolicyService
{
    fn check_user_access(&self, server: &crate::Server, net: &Network, client: &ClientConnection) -> bool;
}
