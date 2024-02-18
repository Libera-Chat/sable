use sable_network::network::event::EventClock;

const EVENT_JSON: &str = r###"
[
  {
    "id": [1,1707606926,2],
    "timestamp": 1707606936,
    "clock": {
      "2": [2,1707606928,0],
      "1": [1,1707606926,1]
    },
    "target": {
      "User": [1,1707606926,1]
    },
    "details": {
      "NewUser": {
        "nickname": "user1",
        "username": "a",
        "visible_hostname": "localhost",
        "realname": "d",
        "mode": {
          "modes": 0
        },
        "server": 1,
        "account": null,
        "initial_connection": [
          [1,1707606926,1],
          {
            "user": [1,1707606926,1],
            "hostname": "localhost",
            "ip": "127.0.0.1",
            "connection_time": 1707606936
          }
        ]
      }
    }
  },
  {
    "id": [1,1707606926,3],
    "timestamp": 1707606949,
    "clock": {
      "2": [2,1707606928,1],
      "1": [1,1707606926,2]
    },
    "target": {
      "Channel": [1,1707606926,1]
    },
    "details": {
      "NewChannel": {
        "name": "#test",
        "mode": {
          "modes": 0,
          "key": null
        }
      }
    }
  },
  {
    "id": [1,1707606926,4],
    "timestamp": 1707606949,
    "clock": {
      "2": [2,1707606928,1],
      "1": [1,1707606926,3]
    },
    "target": {
      "Membership": [[1,1707606926,1],[1,1707606926,1]]
    },
    "details": {
      "ChannelJoin": {
        "channel": [1,1707606926,1],
        "user": [1,1707606926,1],
        "permissions": 1
      }
    }
  },
  {
    "id": [1,1707606926,5],
    "timestamp": 1707606957,
    "clock": {
      "2": [2,1707606928,2],
      "1": [1,1707606926,4]
    },
    "target": {
      "Message": [1,1707606926,1]
    },
    "details": {
      "NewMessage": {
        "source": [1,1707606926,1],
        "target": {
          "Channel": [1,1707606926,1]
        },
        "message_type": "Privmsg",
        "text": "test one"
      }
    }
  },
  {
    "id": [2,1707606928,0],
    "timestamp": 1707606928,
    "clock": {
      "1": [1,1707606926,1]
    },
    "target": {
      "Server": 2
    },
    "details": {
      "NewServer": {
        "epoch": 1707606928,
        "name": "server2.test",
        "ts": 1707606928,
        "flags": {
          "bits": 1
        },
        "version": "sable-0.1.0-453866f0b958c774d3103579d000c0e2ab8a8e2f-dirty"
      }
    }
  },
  {
    "id": [2,1707606928,1],
    "timestamp": 1707606945,
    "clock": {
      "2": [2,1707606928,0],
      "1": [1,1707606926,2]
    },
    "target": {
      "User": [2,1707606928,1]
    },
    "details": {
      "NewUser": {
        "nickname": "user",
        "username": "a",
        "visible_hostname": "localhost",
        "realname": "d",
        "mode": {
          "modes": 0
        },
        "server": 2,
        "account": null,
        "initial_connection": [
          [2,1707606928,1],
          {
            "user": [2,1707606928,1],
            "hostname": "localhost",
            "ip": "127.0.0.1",
            "connection_time": 1707606945
          }
        ]
      }
    }
  },
  {
    "id": [2,1707606928,2],
    "timestamp": 1707606952,
    "clock": {
      "2": [2,1707606928,1],
      "1": [1,1707606926,4]
    },
    "target": {
      "Membership": [[2,1707606928,1],[1,1707606926,1]]
    },
    "details": {
      "ChannelJoin": {
        "channel": [1,1707606926,1],
        "user": [2,1707606928,1],
        "permissions": 0
      }
    }
  },
  {
    "id": [2,1707606928,3],
    "timestamp": 1707606962,
    "clock": {
      "2": [2,1707606928,2],
      "1": [1,1707606926,5]
    },
    "target": {
      "Message": [2,1707606928,1]
    },
    "details": {
      "NewMessage": {
        "source": [2,1707606928,1],
        "target": {
          "Channel": [1,1707606926,1]
        },
        "message_type": "Privmsg",
        "text": "test two"
      }
    }
  }
]
"###;

pub fn sample_events() -> Vec<sable_network::network::event::Event> {
    serde_json::from_str(EVENT_JSON).unwrap()
}

pub fn initial_clock() -> EventClock {
    serde_json::from_str(
        r#"{
        "1": [1,1707606926,1]
      }"#,
    )
    .unwrap()
}
