use irc_network::*;
use crate::connection::*;

use async_std::{
    channel,
    net::IpAddr,
    task
};
use async_std_resolver::AsyncStdResolver;

pub struct DnsClient
{
    event_channel: channel::Sender<ConnectionEvent>,
    resolver: AsyncStdResolver,
}

impl DnsClient
{
    pub fn new(event_channel: channel::Sender<ConnectionEvent>) -> Self
    {
        let resolver = task::block_on(async { async_std_resolver::resolver_from_system_conf().await.unwrap() });
        Self {
            event_channel: event_channel,
            resolver: resolver
        }
    }

    pub fn start_lookup(&self, conn_id: ConnectionId, addr: IpAddr)
    {
        let chan = self.event_channel.clone();
        let resolver = self.resolver.clone();

        task::spawn(async move {
            let name = match resolver.reverse_lookup(addr).await
            {
                Ok(lookup) => lookup.iter().next().map(|n| Hostname::new(n.to_string()).ok()).flatten(),
                Err(_) => None
            };
            let _res = chan.send(ConnectionEvent{source: conn_id, detail: EventDetail::DNSLookupFinished(name)}).await;
        });
    }
}