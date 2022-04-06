use crate::*;
use super::fixtures::*;

#[test]
fn empty_network_can_be_serialized()
{
    let net = Network::new(config::NetworkConfig::new());
    let _str = serde_json::to_string(&net).unwrap();
}

#[test]
fn net_with_channel_can_be_serialized()
{
    let mut builder = NetworkBuilder::new();
    builder.add_channel(ChannelName::from_str("#a").unwrap());
    let str = serde_json::to_string(&builder.net).unwrap();
    //assert_eq!(str, "{\"nick_bindings\":[],\"users\":[],\"user_modes\":[],\"channels\":[[[1,1,1],{\"id\":[1,1,1],\"name\":\"#a\",\"mode\":[1,1,1]}]],\"channel_modes\":[[[1,1,1],{\"id\":[1,1,1],\"modes\":0}]],\"channel_list_modes\":[[[[1,1,1],\"b\"],{\"id\":[[1,1,1],\"b\"],\"list_type\":\"b\"}],[[[1,1,1],\"I\"],{\"id\":[[1,1,1],\"I\"],\"list_type\":\"I\"}],[[[1,1,1],\"q\"],{\"id\":[[1,1,1],\"q\"],\"list_type\":\"q\"}],[[[1,1,1],\"e\"],{\"id\":[[1,1,1],\"e\"],\"list_type\":\"e\"}]],\"list_mode_entries\":[],\"channel_topics\":[],\"memberships\":[],\"messages\":[],\"servers\":[],\"clock\":{\"1\":[1,1,2]}}");
    let net: Network = serde_json::from_str(&str).unwrap();
    assert_eq!(net.channels().count(), 1);
    assert_eq!(net.users().count(), 0);
}

#[test]
fn net_with_user_can_be_serialized()
{
    let mut builder = NetworkBuilder::new();
    builder.add_user(Nickname::from_str("a").unwrap());
    let str = serde_json::to_string(&builder.net).unwrap();
    //assert_eq!(str, "{\"nick_bindings\":[[\"a\",{\"nick\":\"a\",\"user\":[1,1,1],\"timestamp\":0,\"created\":[1,1,2]}]],\"users\":[[[1,1,1],{\"id\":[1,1,1],\"server\":1,\"user\":\"a\",\"visible_host\":\"host.name\",\"realname\":\"user\",\"mode_id\":[1,1,1]}]],\"user_modes\":[[[1,1,1],{\"id\":[1,1,1],\"modes\":0}]],\"channels\":[],\"channel_modes\":[],\"channel_list_modes\":[],\"list_mode_entries\":[],\"channel_topics\":[],\"memberships\":[],\"messages\":[],\"servers\":[],\"clock\":{\"1\":[1,1,2]}}");
    let net: Network = serde_json::from_str(&str).unwrap();
    assert_eq!(net.channels().count(), 0);
    assert_eq!(net.users().count(), 1);
}