use std::net::IpAddr;
use std::ops::Deref;

use ipnet::IpNet;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostMatcher {
    Hostname(Pattern),
    Ip(IpNet),
}

impl HostMatcher {
    pub fn is_host(&self) -> bool {
        matches!(self, Self::Hostname(_))
    }

    pub fn is_ip(&self) -> bool {
        matches!(self, Self::Ip(_))
    }

    pub fn matches_host(&self, hostname: &str) -> bool {
        match self {
            Self::Hostname(pat) => pat.matches(hostname),
            _ => false,
        }
    }

    pub fn matches_ip(&self, addr: &IpAddr) -> bool {
        match self {
            Self::Ip(mask) => mask.contains(addr),
            _ => false,
        }
    }

    pub fn matches(&self, hostname: &str, addr: &IpAddr) -> bool {
        match self {
            Self::Hostname(pat) => pat.matches(hostname),
            Self::Ip(mask) => mask.contains(addr),
        }
    }
}

pub struct UserHostMatcher {
    user: Pattern,
    host: HostMatcher,
}

impl UserHostMatcher {
    pub fn matches(&self, username: &str, hostname: &str, addr: &IpAddr) -> bool {
        self.user.matches(username) && self.host.matches(hostname, addr)
    }
}

pub struct IpMatcher(IpNet);

impl IpMatcher {
    pub fn matches(&self, addr: &IpAddr) -> bool {
        self.0.contains(addr)
    }
}

pub struct ExactIpMatcher(IpAddr);

impl ExactIpMatcher {
    pub fn matches(&self, addr: &IpAddr) -> bool {
        &self.0 == addr
    }
}

pub struct NicknameMatcher(Pattern);

impl Deref for NicknameMatcher {
    type Target = Pattern;

    fn deref(&self) -> &Pattern {
        &self.0
    }
}

impl NicknameMatcher {
    pub fn new(pattern: Pattern) -> NicknameMatcher {
        NicknameMatcher(pattern)
    }

    pub fn matches(&self, nick: &Nickname) -> bool {
        self.0.matches(nick.as_ref())
    }
}
