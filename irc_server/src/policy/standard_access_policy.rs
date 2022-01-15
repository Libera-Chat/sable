use super::*;
use crate::PreClient;
use crate::numeric;

pub struct StandardAccessPolicy
{
}

impl StandardAccessPolicy
{
    pub fn new() -> Self
    {
        Self {
        }
    }

    pub fn find_kline<'a>(&self, net: &'a Network, client: &PreClient) -> Option<wrapper::KLine<'a>>
    {
        if let (Some(user), Some(host)) = (client.user, client.hostname)
        {
            for kline in net.klines()
            {
                if kline.user().matches(&user.value()) && kline.host().matches(&host.value())
                {
                    return Some(kline);
                }
            }
        }
        None
    }
}

impl AccessPolicyService for StandardAccessPolicy
{
    fn check_user_access(&self, server: &crate::Server, net: &Network, client: &ClientConnection) -> bool
    {
        if let Some(pre_client) = &client.pre_client
        {
            let pre_client: &PreClient = &pre_client.borrow();
            if let Some(kline) = self.find_kline(net, pre_client)
            {
                client.send(&numeric::YoureBanned::new(kline.reason()).format_for(server, pre_client));
                return false;
            }
        }
        true
    }
}