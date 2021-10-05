pub mod ircd;

use ircd::event::{EventDetails};
use ircd::event::details::*;

static SERVER_ID: i64 = 1;

fn main() {
    let mut net = ircd::Network::new();
    let mut log = ircd::event::EventLog::new(SERVER_ID);
    let mut idgen = ircd::IdGenerator::new(SERVER_ID);

    let mut consumer = log.get_offset();

    let user_id = idgen.next();
    let e = log.create(user_id, EventDetails::NewUser(NewUser {
            nickname: "aaa".to_string(),
            username: "aaa".to_string(),
            visible_hostname: "example.com".to_string()
        }));

    log.add(e);

    let channel_id = idgen.next();
    let e = log.create(channel_id, EventDetails::NewChannel(NewChannel { name: "##".to_string() }));
    log.add(e);

    let e = log.create(idgen.next(), EventDetails::ChannelJoin(ChannelJoin { user: user_id, channel: channel_id }));
    log.add(e);

    while let Some(event) = log.next_for(&mut consumer) {
        net.apply(&event);
    }

    for u in net.users() {
        println!("got user: {}!{}@{}", u.nick(), u.user(), u.visible_host());
    }

    for c in net.channels() {
        println!("got channel: {}", c.name());
    }

    for m in net.memberships() {
        println!("got membership: {:?}/{} in {:?}/{}",
            m.user_id(), m.user().unwrap().nick(),
            m.channel_id(), m.channel().unwrap().name()
        );
    }
}
