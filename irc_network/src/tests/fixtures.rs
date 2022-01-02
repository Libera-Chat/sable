use crate::*;
use event::*;

pub struct NetworkBuilder
{
    pub net: Network,
    id_gen: ObjectIdGenerator,
}

struct NopUpdateReceiver;

impl NetworkUpdateReceiver for NopUpdateReceiver
{
    fn notify_update(&self, _update: NetworkStateChange)
    { }
}

impl NetworkBuilder
{
    pub fn new() -> Self
    {
        Self {
            net: Network::new(),
            id_gen: ObjectIdGenerator::new(ServerId::new(1), EpochId::new(1)),
        }
    }

    pub fn add_channel(&mut self, name: ChannelName)
    {
        let mode_id = self.id_gen.next_channel_mode();

        let evt = Event {
            clock: EventClock::new(),
            id: self.id_gen.next_event(),
            target: mode_id.into(),
            timestamp: 0,
            details: details::NewChannelMode {
                mode: ChannelModeSet::new()
            }.into()
        };
        self.net.apply(&evt, &NopUpdateReceiver).unwrap();

        let evt = Event { 
            clock: EventClock::new(),
            id: self.id_gen.next_event(),
            target: self.id_gen.next_channel().into(),
            timestamp: 0,
            details: details::NewChannel {
                mode: mode_id,
                name: name
            }.into()
        };
        self.net.apply(&evt, &NopUpdateReceiver).unwrap();
    }

    pub fn add_user(&mut self, nick: Nickname)
    {
        let mode_id = self.id_gen.next_user_mode();

        let evt = Event {
            clock: EventClock::new(),
            id: self.id_gen.next_event(),
            target: mode_id.into(),
            timestamp: 0,
            details: details::NewUserMode {
                mode: UserModeSet::new()
            }.into()
        };
        self.net.apply(&evt, &NopUpdateReceiver).unwrap();

        let evt = Event { 
            clock: EventClock::new(),
            id: self.id_gen.next_event(),
            target: self.id_gen.next_user().into(),
            timestamp: 0,
            details: details::NewUser {
                mode_id: mode_id,
                nickname: nick,
                username: Username::from_str("a").unwrap(),
                realname: "user".to_string(),
                visible_hostname: Hostname::from_str("host.name").unwrap(),
                server: ServerId::new(1),
            }.into()
        };
        self.net.apply(&evt, &NopUpdateReceiver).unwrap();
    }
}