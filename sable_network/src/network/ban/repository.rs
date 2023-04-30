use crate::network::*;
use super::*;

use std::collections::HashMap;
use std::net::IpAddr;

/// A collection of network bans, supporting efficient lookup based on
/// (partial) user details
#[derive(Debug,Clone)]
pub struct BanRepository
{
    all_bans: HashMap<NetworkBanId, state::NetworkBan>,

    exact_host_bans: HashMap<Hostname, Vec<NetworkBanId>>,
    exact_ip_bans: HashMap<IpAddr, Vec<NetworkBanId>>,

    host_range_bans: HashMap<String, Vec<NetworkBanId>>,
    ip_net_bans: HashMap<IpAddr, Vec<NetworkBanId>>,

    // Freeform host bans aren't indexable and just have to all be tested
    freeform_hostmask_bans: Vec<NetworkBanId>,
}

impl BanRepository
{
    pub fn new() -> Self
    {
        Self {
            all_bans: HashMap::new(),
            exact_host_bans: HashMap::new(),
            exact_ip_bans: HashMap::new(),
            host_range_bans: HashMap::new(),
            ip_net_bans: HashMap::new(),
            freeform_hostmask_bans: Vec::new(),
        }
    }

    pub fn from_ban_set(bans: Vec<state::NetworkBan>) -> Result<Self, DuplicateNetworkBan>
    {
        let mut ret = Self::new();

        for ban in bans
        {
            ret.add(ban)?;
        }

        Ok(ret)
    }

    fn add_index_for(&mut self, ban: &state::NetworkBan) -> Result<(), NetworkBanId>
    {
        let ban_vec = match &ban.matcher.host
        {
            NetworkBanHostMatch::ExactIp(ip) => self.exact_ip_bans.entry(*ip).or_default(),
            NetworkBanHostMatch::IpRange(ip_net) => self.ip_net_bans.entry(ip_net.network()).or_default(),
            NetworkBanHostMatch::ExactHostname(host) => self.exact_host_bans.entry(*host).or_default(),
            NetworkBanHostMatch::HostnameRange(host_suffix) =>self.host_range_bans.entry(host_suffix.clone()).or_default(),
            NetworkBanHostMatch::HostnameMask(_) => &mut self.freeform_hostmask_bans
        };

        // We don't strictly need to do this if we just created the vec, but it won't take
        // long to iterate an empty vector and the code's clearer this way.

        // We can't use `self` in the closure because it's already borrowed mutably; declaring this
        // here lets the closure access a single field
        let all_bans = &self.all_bans;
        if let Some(existing) = ban_vec.iter().find(
            |id| {
                if let Some(other_ban) = all_bans.get(id) {
                    ban.matcher == other_ban.matcher
                } else {
                    false
                }
            })
        {
            Err(*existing)
        }
        else
        {
            ban_vec.push(ban.id);
            Ok(())
        }
    }

    pub fn add(&mut self, ban: state::NetworkBan) -> Result<(), DuplicateNetworkBan>
    {
        if let Err(existing_id) = self.add_index_for(&ban)
        {
            Err(DuplicateNetworkBan { existing_id, ban })
        }
        else
        {
            self.all_bans.insert(ban.id, ban);
            Ok(())
        }
    }

    pub fn remove(&mut self, id: NetworkBanId)
    {
        if let Some(ban) = self.all_bans.remove(&id)
        {
            let search_vec = match &ban.matcher.host
            {
                NetworkBanHostMatch::ExactIp(ip) => self.exact_ip_bans.get_mut(&ip),
                NetworkBanHostMatch::IpRange(ip_net) => self.ip_net_bans.get_mut(&ip_net.network()),
                NetworkBanHostMatch::ExactHostname(host) => self.exact_host_bans.get_mut(host),
                NetworkBanHostMatch::HostnameRange(host) => self.host_range_bans.get_mut(host),
                NetworkBanHostMatch::HostnameMask(_) => Some(&mut self.freeform_hostmask_bans),
            };
            if let Some(search_vec) = search_vec
            {
                search_vec.retain(|id| id != &ban.id)
            }
        }
    }

    pub fn get(&self, id: &NetworkBanId) -> Option<&state::NetworkBan>
    {
        self.all_bans.get(id)
    }

    pub fn find(&self, user_details: &UserDetails) -> Option<&state::NetworkBan>
    {
        let mut candidates = Vec::new();

        // First look for an exact IP match
        if let Some(vec) = user_details.ip.and_then(|ip| self.exact_ip_bans.get(ip))
        {
            candidates.push(vec);
        }

        // Then an exact hostname match
        if let Some(vec) = user_details.host.and_then(|host| self.exact_host_bans.get(host))
        {
            candidates.push(vec);
        }

        // Then run through each prefix length checking for range bans
        if let Some(ip) = user_details.ip
        {
            let mut next_net: Option<IpNet> = Some((*ip).into());

            while let Some(ref net) = next_net
            {
                if let Some(vec) = self.ip_net_bans.get(&net.network())
                {
                    candidates.push(vec)
                }
                next_net = net.supernet();
            }
        }

        // Then go through each possible hostname suffix
        if let Some(host) = user_details.host
        {
            let mut host_part = host;

            while let Some((_, suffix)) = host_part.split_once('.')
            {
                if let Some(vec) = self.host_range_bans.get(suffix)
                {
                    candidates.push(vec);
                }
                host_part = suffix;
            }
        }

        candidates.push(&self.freeform_hostmask_bans);

        // `candidates` is now ordered: exact IP match first, then exact hostname,
        // then CIDR range bans from most specific to least specific prefix length,
        // then hostname suffix bans from most to least specific, then freeform mask bans.
        //
        // The first of these that matches on its other criteria is the one we'll use.
        for id in candidates.into_iter().flatten()
        {
            if let Some(candidate_ban) = self.all_bans.get(&id)
            {
                if candidate_ban.matcher.matches(user_details)
                {
                    return Some(candidate_ban)
                }
            }
        }

        None
    }
}

impl serde::ser::Serialize for BanRepository
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        serializer.collect_seq(self.all_bans.values())
    }
}

impl<'de> serde::de::Deserialize<'de> for BanRepository
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        let bans = Vec::deserialize(deserializer)?;
        Self::from_ban_set(bans).map_err(serde::de::Error::custom)
    }
}