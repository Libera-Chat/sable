use super::*;

impl ClientServer
{
    pub(super) fn find_kline<'a>(&self, net: &'a Network, client: &PreClient) -> Option<wrapper::KLine<'a>>
    {
        if let (Some(user), Some(host)) = (client.user.get(), client.hostname.get())
        {
            for kline in net.klines()
            {
                if kline.user().matches(user.value()) && kline.host().matches(host.value())
                {
                    return Some(kline);
                }
            }
        }
        None
    }

    pub(super) fn check_user_access(&self, server: &crate::ClientServer, net: &Network, client: &ClientConnection) -> bool
    {
        if let Some(pre_client) = &client.pre_client
        {
            if let Some(kline) = self.find_kline(net, pre_client)
            {
                client.send(&numeric::YoureBanned::new(kline.reason()).format_for(server, &UnknownTarget));
                return false;
            }
        }
        true
    }
}