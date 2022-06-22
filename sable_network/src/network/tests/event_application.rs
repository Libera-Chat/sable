use crate::prelude::*;
use super::fixtures::*;

#[test]
fn add_and_remove_user()
{
    let mut builder = NetworkBuilder::new();
    let empty_net = builder.json_for_compare();
    let nick = Nickname::from_str("aaa").unwrap();
    builder.add_user(nick);
    let user_id = builder.net.user_by_nick(&nick).unwrap().id();
    builder.remove_user(user_id);
    let modified_net = builder.json_for_compare();

    assert_eq!(empty_net, modified_net);
}
