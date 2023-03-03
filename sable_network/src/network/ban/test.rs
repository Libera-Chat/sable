use std::net::{
    Ipv4Addr,
    Ipv6Addr,
};

use super::*;

#[test]
fn parse_ip4_exact()
{
    let match1 = NetworkBanHostMatch::from_str("192.168.0.1").unwrap();
    let expected = NetworkBanHostMatch::ExactIp(Ipv4Addr::new(192, 168, 0, 1).into());
    assert_eq!(match1, expected);
}

#[test]
fn parse_ip4_wildcard()
{
    let match1 = NetworkBanHostMatch::from_str("192.168.0.*").unwrap();
    let ip = Ipv4Addr::new(192, 168, 0, 0).into();
    let expected = NetworkBanHostMatch::IpRange(IpNet::new(ip, 24).unwrap());
    assert_eq!(match1, expected);
}

#[test]
fn parse_ip4_cidr()
{
    let match1 = NetworkBanHostMatch::from_str("192.168.0.0/24").unwrap();
    let ip = Ipv4Addr::new(192, 168, 0, 0).into();
    let expected = NetworkBanHostMatch::IpRange(IpNet::new(ip, 24).unwrap());
    assert_eq!(match1, expected);
}

#[test]
fn parse_ip6_exact()
{
    let match1 = NetworkBanHostMatch::from_str("::1").unwrap();
    let expected = NetworkBanHostMatch::ExactIp(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1).into());
    assert_eq!(match1, expected);
}

#[test]
fn parse_ip6_wildcard()
{
    let match1 = NetworkBanHostMatch::from_str("fc00:1:2:3:*").unwrap();
    let ip = Ipv6Addr::new(0xfc00, 1, 2, 3, 0, 0, 0, 0).into();
    let expected = NetworkBanHostMatch::IpRange(IpNet::new(ip, 64).unwrap());
    assert_eq!(match1, expected);
}

#[test]
fn parse_ip6_cidr()
{
    let match1 = NetworkBanHostMatch::from_str("fc00:1:2:3::/64").unwrap();
    let ip = Ipv6Addr::new(0xfc00, 1, 2, 3, 0, 0, 0, 0).into();
    let expected = NetworkBanHostMatch::IpRange(IpNet::new(ip, 64).unwrap());
    assert_eq!(match1, expected);
}

#[test]
fn parse_hostname_exact()
{
    let match1 = NetworkBanHostMatch::from_str("foo.bar.example.com").unwrap();
    let host = Hostname::from_str("foo.bar.example.com").unwrap();
    let expected = NetworkBanHostMatch::ExactHostname(host);
    assert_eq!(match1, expected);
}

#[test]
fn parse_hostname_suffix()
{
    let match1 = NetworkBanHostMatch::from_str("*.bar.example.com").unwrap();
    let host = "bar.example.com".to_owned();
    let expected = NetworkBanHostMatch::HostnameRange(host);
    assert_eq!(match1, expected);
}

#[test]
fn parse_hostname_freeform()
{
    let match1 = NetworkBanHostMatch::from_str("foo.*.example.com").unwrap();
    let host = Pattern::new("foo.*.example.com".to_owned());
    let expected = NetworkBanHostMatch::HostnameMask(host);
    assert_eq!(match1, expected);
}