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

    pub fn json_for_compare(&self) -> serde_json::Value
    {
        let mut json = serde_json::to_value(&self.net).unwrap();
        json.as_object_mut().unwrap().remove("clock");
        json
    }

    fn apply(&mut self, target: impl Into<ObjectId>, details: impl Into<EventDetails>)
    {
        let evt = Event {
            clock: EventClock::new(),
            id: self.id_gen.next_event(),
            target: target.into(),
            timestamp: 0,
            details: details.into()
        };
        self.net.apply(&evt, &NopUpdateReceiver).unwrap();
    }

    pub fn add_channel(&mut self, name: ChannelName)
    {
        let mode_id = self.id_gen.next_channel_mode();

        self.apply(mode_id, details::NewChannelMode { mode: ChannelModeSet::new() });

        self.apply(self.id_gen.next_channel(), details::NewChannel {
                mode: mode_id,
                name: name
            });
    }

    pub fn add_user(&mut self, nick: Nickname)
    {
        let mode_id = self.id_gen.next_user_mode();

        self.apply(mode_id, details::NewUserMode {
                mode: UserModeSet::new()
            });

        self.apply(self.id_gen.next_user(), details::NewUser {
                mode_id: mode_id,
                nickname: nick,
                username: Username::from_str("a").unwrap(),
                realname: "user".to_string(),
                visible_hostname: Hostname::from_str("host.name").unwrap(),
                server: ServerId::new(1),
            });
    }

    pub fn remove_user(&mut self, id: UserId)
    {
        self.apply(id, details::UserQuit { message: "quit".to_string() })
    }
}