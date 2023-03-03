use crate::{
    id::*,
    validated::*,
    types::Pattern,
};

use std::{
    str::FromStr,
    convert::TryInto
};
use ipnet::IpNet;
use serde::{Serialize,Deserialize};
use thiserror::Error;

mod user_details;
pub use user_details::*;

mod repository;
pub use repository::*;

/// Actions that can be applied by a network ban
#[derive(PartialEq,Debug,Clone,Serialize,Deserialize)]
pub enum NetworkBanAction
{
    /// Refuse new connections that match these criteria. The boolean parameter
    /// determines whether existing connections that match will also be disconnected.
    RefuseConnection(bool),
    /// Require that new connections matching these criteria log in to an account
    /// before registration. The boolean parameter determines whether existing matching
    /// connections that are not logged in to an account will be disconnected.
    RequireSasl(bool),
    /// Refuse new connections instantly, without allowing exemptions from other config entries
    /// (equivalent to legacy D:line). Only makes sense for a ban that matches only on
    /// IP address; the other information won't be present at immediate-disconnection time.
    DisconnectEarly,
}

/// Methods by which a network ban can match a user's host
#[derive(PartialEq,Debug,Clone,Serialize,Deserialize)]
pub enum NetworkBanHostMatch
{
    /// Exact IP address match
    ExactIp(std::net::IpAddr),
    /// IP address range (i.e. CIDR mask)
    IpRange(ipnet::IpNet),
    /// Exact resolved hostname match
    ExactHostname(Hostname),
    /// Resolved hostname range (i.e. *.suffix)
    HostnameRange(String),
    /// Freeform pattern (e.g. ip-8-8-*.dynamic-isp.com, 192.*.*.1)
    HostnameMask(Pattern)
}

/// Error type denoting an invalid ban mask was supplied
#[derive(Debug,Clone,Error)]
#[error("Invalid ban mask")]
pub struct InvalidBanMask;

/// Error type denoting that a duplicate ban was provided
#[derive(Debug,Clone,Error)]
#[error("Duplicate network ban")]
pub struct DuplicateNetworkBan(
    /// The ID of the pre-existing ban
    pub NetworkBanId
);

/// Criteria for matching a network ban.
///
/// All fields except host are optional, and the combination will match if all its
/// fields that are present match individually.
#[derive(PartialEq,Debug,Clone,Serialize,Deserialize)]
pub struct NetworkBanMatch
{
    pub host: NetworkBanHostMatch,
    pub ident: Option<Pattern>,
    pub realname: Option<Pattern>,
    pub nickname: Option<Pattern>,
}

impl FromStr for NetworkBanHostMatch
{
    type Err = InvalidBanMask;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        let ipv4_chars = "0123456789.";
        let ipv6_chars = "0123456789abcdef:";
        let wildcard_chars = "*?";

        if let Ok(ip) = std::net::IpAddr::from_str(s)
        {
            Ok(Self::ExactIp(ip))
        }
        else if let Ok(ip_net) = ipnet::IpNet::from_str(s)
        {
            Ok(Self::IpRange(ip_net))
        }
        else if s.chars().any(|c| wildcard_chars.contains(c))
        {
            // There's a wildcard involved somewhere. *.host.name and 1.2.* or aa:bb:*
            // patterns are handled specially, so look for those

            if s.ends_with(".*") &&
                s.split_at(s.len() - 1).0.chars().all(|c| ipv4_chars.contains(c))
            {
                // This is an IPv4 prefix mask, which we can turn into a CIDR-based IpNet

                let mut components = s.split('.').collect::<Vec<_>>();

                if components.len() > 4
                {
                    return Err(InvalidBanMask);
                }

                // We know from the surrounding if that the last component is *
                components.pop();
                let prefix_len = (components.len() * 8) as u8; // Can't overflow because of length test above

                let mut numbers = Vec::<u8>::new();
                for c in components
                {
                    numbers.push(c.parse().map_err(|_| InvalidBanMask)?);
                }
                while numbers.len() < 4 { numbers.push(0); }

                let ip_addr = std::net::Ipv4Addr::new(numbers[0], numbers[1], numbers[2], numbers[3]);
                let ip_net = IpNet::new(ip_addr.into(), prefix_len).map_err(|_| InvalidBanMask)?;

                Ok(Self::IpRange(ip_net))
            }
            else if s.ends_with(":*") &&
                s.split_at(s.len() - 1).0.chars().all(|c| ipv6_chars.contains(c))
            {
                // This is an IPv6 prefix mask, which we can turn into a CIDR-based IpNet

                let mut components = s.split(':').collect::<Vec<_>>();

                if components.len() > 8
                {
                    return Err(InvalidBanMask);
                }

                // We know from the surrounding if that the last component is *
                components.pop();
                let prefix_len = (components.len() * 16) as u8; // Can't overflow because of length test above

                let mut numbers = Vec::<u16>::new();
                for c in components
                {
                    numbers.push(u16::from_str_radix(c, 16).map_err(|_| InvalidBanMask)?);
                }
                while numbers.len() < 8 { numbers.push(0); }

                let ip_addr = std::net::Ipv6Addr::new(numbers[0], numbers[1], numbers[2], numbers[3], numbers[4], numbers[5], numbers[6], numbers[7]);
                let ip_net = IpNet::new(ip_addr.into(), prefix_len).map_err(|_| InvalidBanMask)?;

                Ok(Self::IpRange(ip_net))
            }
            else if s.starts_with("*.") && ! s[2..].chars().any(|c| wildcard_chars.contains(c))
            {
                // If the first character is * and there are no other wildcards, then
                // it's a suffix-based range
                Ok(Self::HostnameRange(s[2..].to_owned()))
            }
            else
            {
                // If we didn't match any of the above, treat it as a free-form wildcard pattern
                // that's not amenable to quick lookup
                Ok(Self::HostnameMask(Pattern::new(s.to_owned())))
            }
        }
        else
        {
            Ok(Self::ExactHostname(s.try_into().map_err(|_| InvalidBanMask)?))
        }
    }
}

impl NetworkBanHostMatch
{
    pub fn matches(&self, user_details: &UserDetails) -> bool
    {
        match self
        {
            NetworkBanHostMatch::ExactIp(ip) =>
            {
                user_details.ip == Some(ip)
            }
            NetworkBanHostMatch::IpRange(ip_net) =>
            {
                if let Some(ip) = user_details.ip {
                    ip_net.contains(ip)
                } else {
                    false
                }
            }
            NetworkBanHostMatch::ExactHostname(host) =>
            {
                user_details.host == Some(host.as_ref())
            }
            NetworkBanHostMatch::HostnameRange(host_suffix) =>
            {
                if let Some(host) = user_details.host {
                    host.ends_with(host_suffix)
                } else {
                    false
                }
            }
            NetworkBanHostMatch::HostnameMask(mask) =>
            {
                let host_match = if let Some(host) = user_details.host {
                    mask.matches(host)
                } else {
                    false
                };

                let ip_match = if let Some(ip) = user_details.ip {
                    mask.matches(&ip.to_string())
                } else {
                    false
                };

                host_match || ip_match
            }
        }
    }
}

impl NetworkBanMatch
{
    pub fn from_user_host(user: &str, host: &str) -> Result<Self, InvalidBanMask>
    {
        let ident_match = if user == "*" { None } else { Some(Pattern::new(user.to_owned())) };

        let host_match = NetworkBanHostMatch::from_str(host)?;

        Ok(Self {
            host: host_match,
            ident: ident_match,
            realname: None,
            nickname: None,
        })
    }

    pub fn matches(&self, user_details: &UserDetails) -> bool
    {
        if ! self.host.matches(user_details)
        {
            return false;
        }

        if let (Some(ident), Some(user_ident)) = (self.ident.as_ref(), user_details.ident)
        {
            if ! ident.matches(user_ident)
            {
                return false;
            }
        }
        if let (Some(realname), Some(user_realname)) = (self.realname.as_ref(), user_details.realname)
        {
            if ! realname.matches(user_realname.as_ref())
            {
                return false;
            }
        }
        if let (Some(nickname), Some(user_nickname)) = (self.nickname.as_ref(), user_details.nick)
        {
            if ! nickname.matches(user_nickname.as_ref())
            {
                return false;
            }
        }

        return true;
    }
}

#[cfg(test)]
mod test;