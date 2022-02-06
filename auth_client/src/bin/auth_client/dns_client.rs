use auth_client::*;
use irc_network::*;
use client_listener::ConnectionId;

use tokio::{
    sync::mpsc::{
        UnboundedSender,
    },
    task
};
use std::net::IpAddr;
use trust_dns_resolver::TokioAsyncResolver;

pub struct InternalDnsClient
{
    event_channel: UnboundedSender<DnsResult>,
    resolver: TokioAsyncResolver,
}

impl InternalDnsClient
{
    pub fn new(event_channel: UnboundedSender<DnsResult>) -> Self
    {
        let resolver = TokioAsyncResolver::tokio_from_system_conf().expect("Failed to create DNS resolver");
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
                Ok(lookup) => {
                    match lookup.iter().next() {
                        Some(name) => {
                            if Self::verify_forward_matches(&resolver, name, addr).await {
                                Some(name.to_ascii())
                            } else {
                                None
                            }
                        },
                        None => None
                    }
                },
                Err(_) => None
            };
            let name = name.map(|n| Hostname::convert(n.trim_end_matches('.')).ok()).flatten();
            let _res = chan.send(DnsResult { conn: conn_id, hostname: name});
        });
    }

    async fn verify_forward_matches(resolver: &TokioAsyncResolver, name: &trust_dns_resolver::Name, addr: IpAddr) -> bool
    {
        match resolver.lookup_ip(name.clone()).await
        {
            Ok(lookup) => {
                lookup.iter().any(|ip| ip == addr)
            },
            Err(_) => false
        }
    }
}