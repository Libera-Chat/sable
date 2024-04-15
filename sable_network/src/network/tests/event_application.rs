use super::fixtures::*;
use crate::prelude::*;
use serde_json::Value;
use std::str::FromStr;

#[test]
fn add_and_remove_user() {
    let mut builder = NetworkBuilder::new();
    let empty_net = builder.json_for_compare();
    let nick = Nickname::from_str("aaa").unwrap();
    builder.add_user(nick);
    let user_id = builder.net.user_by_nick(&nick).unwrap().id();
    builder.remove_user(user_id);
    let mut modified_net = builder.json_for_compare();

    if let Value::Object(map) = &mut modified_net {
        // Empty this array, because adding and removing a user changes it.
        map["historic_nick_users"] = Value::Array(Vec::new());
    }

    assert_eq!(empty_net, modified_net);
}
