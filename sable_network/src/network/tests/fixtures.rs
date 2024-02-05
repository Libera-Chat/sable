use crate::prelude::*;
use event::*;
use std::str::FromStr;

pub struct NetworkBuilder {
    pub net: Network,
    id_gen: ObjectIdGenerator,
}

struct NopUpdateReceiver;

impl NetworkUpdateReceiver for NopUpdateReceiver {
    fn notify_update(&self, _update: NetworkStateChange, _event: &Event) {}
}

impl NetworkBuilder {
    pub fn new() -> Self {
        Self {
            net: Network::new(config::NetworkConfig::new()),
            id_gen: ObjectIdGenerator::new(ServerId::new(1), EpochId::new(1)),
        }
    }

    pub fn json_for_compare(&self) -> serde_json::Value {
        let mut json = serde_json::to_value(&self.net).unwrap();
        json.as_object_mut().unwrap().remove("clock");
        json
    }

    fn apply(&mut self, target: impl Into<ObjectId>, details: impl Into<EventDetails>) {
        let evt = Event {
            clock: EventClock::new(),
            id: self.id_gen.next_event(),
            target: target.into(),
            timestamp: 0,
            details: details.into(),
        };
        self.net.apply(&evt, &NopUpdateReceiver).unwrap();
    }

    pub fn add_channel(&mut self, name: ChannelName) {
        self.apply(
            self.id_gen.next_channel(),
            details::NewChannel {
                mode: state::ChannelMode::new(ChannelModeSet::default()),
                name: name,
            },
        );
    }

    pub fn add_user(&mut self, nick: Nickname) {
        self.apply(
            self.id_gen.next_user(),
            details::NewUser {
                mode: state::UserMode::new(UserModeSet::default()),
                nickname: nick,
                username: Username::from_str("a").unwrap(),
                realname: Realname::from_str("user").unwrap(),
                visible_hostname: Hostname::from_str("host.name").unwrap(),
                server: ServerId::new(1),
                account: None,
                initial_connection: None,
            },
        );
    }

    pub fn remove_user(&mut self, id: UserId) {
        self.apply(
            id,
            details::UserQuit {
                message: "quit".to_string(),
            },
        )
    }
}
