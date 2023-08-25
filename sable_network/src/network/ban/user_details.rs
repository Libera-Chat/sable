use crate::network::wrapper;
use std::net::IpAddr;

/// Details of a user to be used when looking up a network ban
#[derive(Default, Clone, Copy)]
pub struct UserDetails<'a, 'b, 'c, 'd, 'e> {
    pub nick: Option<&'a str>,
    pub ident: Option<&'b str>,
    pub host: Option<&'c str>,
    pub ip: Option<&'d IpAddr>,
    pub realname: Option<&'e str>,
}

impl<'a> UserDetails<'a, 'a, 'a, 'a, 'a> {
    /// Construct a `UserDetails` containing only an IP address, for early IP-ban
    /// checking.
    pub fn from_ip(ip: &'a IpAddr) -> Self {
        Self {
            nick: None,
            ident: None,
            host: None,
            ip: Some(ip),
            realname: None,
        }
    }

    /// Construct a `UserDetails` with the ident, visible hostname and realname of an existing user
    pub fn from_user(user: &'a wrapper::User<'a>) -> Self {
        Self {
            nick: None,
            ident: Some(user.user().as_ref()),
            host: Some(user.visible_host().as_ref()),
            ip: None,
            realname: Some(user.realname().as_ref()),
        }
    }
}

impl<'a, 'b, 'c, 'd, 'e> UserDetails<'a, 'b, 'c, 'd, 'e> {
    /// Return a `UserDetails` with the given new nickname, and all other fields taken from `self`
    pub fn with_nick<'new_a>(self, new_nick: &'new_a str) -> UserDetails<'new_a, 'b, 'c, 'd, 'e> {
        UserDetails {
            nick: Some(new_nick),
            ident: self.ident,
            host: self.host,
            ip: self.ip,
            realname: self.realname,
        }
    }

    /// Return a `UserDetails` with the given new ident, and all other fields taken from `self`
    pub fn with_ident<'new_b>(self, new_ident: &'new_b str) -> UserDetails<'a, 'new_b, 'c, 'd, 'e> {
        UserDetails {
            nick: self.nick,
            ident: Some(new_ident),
            host: self.host,
            ip: self.ip,
            realname: self.realname,
        }
    }

    /// Return a `UserDetails` with the given new hostname, and all other fields taken from `self`
    pub fn with_host<'new_c>(self, new_host: &'new_c str) -> UserDetails<'a, 'b, 'new_c, 'd, 'e> {
        UserDetails {
            nick: self.nick,
            ident: self.ident,
            host: Some(new_host),
            ip: self.ip,
            realname: self.realname,
        }
    }

    /// Return a `UserDetails` with the given new IP address, and all other fields taken from `self`
    pub fn with_ip<'new_d>(self, new_ip: &'new_d IpAddr) -> UserDetails<'a, 'b, 'c, 'new_d, 'e> {
        UserDetails {
            nick: self.nick,
            ident: self.ident,
            host: self.host,
            ip: Some(new_ip),
            realname: self.realname,
        }
    }

    /// Return a `UserDetails` with the given new realname, and all other fields taken from `self`
    pub fn with_realname<'new_e>(
        self,
        new_realname: &'new_e str,
    ) -> UserDetails<'a, 'b, 'c, 'd, 'new_e> {
        UserDetails {
            nick: self.nick,
            ident: self.ident,
            host: self.host,
            ip: self.ip,
            realname: Some(new_realname),
        }
    }
}
