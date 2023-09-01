use client_listener::ConnectionId;

/// A message tag attached to an inbound (client->server) message
#[derive(Debug)]
pub struct InboundMessageTag {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug)]
pub struct InboundTagSet(pub Vec<InboundMessageTag>);

impl InboundTagSet {
    pub fn has(&self, name: &str) -> Option<&InboundMessageTag> {
        self.0.iter().find(|t| t.name == name)
    }
}

/// A tokenised, but not yet processed, message from a client connection
#[derive(Debug)]
pub struct ClientMessage {
    /// The connection from which the message was received
    pub source: ConnectionId,
    /// The command
    pub command: String,
    /// The list of arguments
    pub args: Vec<String>,
    /// The list of tags attached to the message
    pub tags: InboundTagSet,
}

impl ClientMessage {
    /// Create a `ClientMessage` from a received message
    pub fn parse(source: ConnectionId, raw: &str) -> Option<Self> {
        let mut args = Vec::new();
        let mut tags = Vec::new();

        let mut raw = raw.trim_start();
        if raw.is_empty() {
            return None;
        }

        if raw.starts_with('@') {
            // We've got message tags, so parse them
            let Some(space_offset) = raw.find(' ') else {
                // TODO: handle this better? We got a string of tags with no command
                return None;
            };

            // Take the text between the '@' and the delimiting space and split
            for tag_def in raw[1..space_offset].split(';') {
                let (name, value) = match tag_def.split_once('=') {
                    Some((n, v)) => (n.to_string(), Some(v.to_string())),
                    None => (tag_def.to_string(), None),
                };

                tags.push(InboundMessageTag { name, value });
            }

            // Skip over the tag definitions and the delimiting space(s)
            raw = raw[space_offset..].trim_start();
        }

        let offset = match raw.find(' ') {
            Some(offset) => offset,
            None => {
                return Some(Self {
                    source,
                    command: raw.to_string(),
                    args: Vec::new(),
                    tags: InboundTagSet(tags),
                });
            }
        };

        let command = &raw[0..offset];
        let mut rest = &raw[offset + 1..];

        loop {
            if let Some(arg) = rest.strip_prefix(':') {
                args.push(arg.to_string());
                break;
            }

            match rest.find(' ') {
                Some(offset) => {
                    let arg = &rest[0..offset];

                    if !arg.is_empty() {
                        args.push(arg.to_string());
                    }

                    rest = &rest[offset + 1..];
                }
                None => {
                    if !rest.is_empty() {
                        args.push(rest.to_string());
                    }
                    break;
                }
            }
        }

        Some(Self {
            source,
            command: command.to_string(),
            args,
            tags: InboundTagSet(tags),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use client_listener::*;

    fn get_connid() -> ConnectionId {
        let listener_id = ListenerIdGenerator::new(0).next();
        ConnectionIdGenerator::new(listener_id, 0).next()
    }

    #[test]
    fn no_args() {
        let msg = ClientMessage::parse(get_connid(), "command").unwrap();
        assert_eq!(msg.command, "command");
        assert_eq!(msg.args.len(), 0);
    }

    #[test]
    fn simple_args() {
        let msg = ClientMessage::parse(get_connid(), "command arg1 arg2 :arg three").unwrap();

        assert_eq!(msg.command, "command");
        assert_eq!(msg.args, &["arg1", "arg2", "arg three"]);
    }

    #[test]
    fn ending_space() {
        let msg = ClientMessage::parse(get_connid(), "command arg1 arg2 ").unwrap();

        assert_eq!(msg.args, &["arg1", "arg2"]);
    }

    #[test]
    fn ending_colon() {
        let msg = ClientMessage::parse(get_connid(), "command arg1 arg2 :").unwrap();

        assert_eq!(msg.args, &["arg1", "arg2", ""]);
    }

    #[test]
    fn double_space() {
        let msg = ClientMessage::parse(get_connid(), "command arg1  arg2").unwrap();

        assert_eq!(msg.args, &["arg1", "arg2"]);
    }

    #[test]
    fn colon_space() {
        let msg = ClientMessage::parse(get_connid(), "command arg1 : arg2").unwrap();

        assert_eq!(msg.args, &["arg1", " arg2"]);
    }

    #[test]
    fn empty() {
        assert!(ClientMessage::parse(get_connid(), "").is_none());
    }

    #[test]
    fn leading_space() {
        let msg = ClientMessage::parse(get_connid(), "    command arg1 arg2 :arg three").unwrap();

        assert_eq!(msg.command, "command");
        assert_eq!(msg.args, &["arg1", "arg2", "arg three"]);
    }

    #[test]
    fn tags() {
        let msg =
            ClientMessage::parse(get_connid(), "@tag1;tag2=val2 command arg1 arg2 :arg three")
                .unwrap();

        assert_eq!(msg.command, "command");
        assert_eq!(msg.args, &["arg1", "arg2", "arg three"]);
        assert_eq!(msg.tags.0.len(), 2);
        println!("{:?}", msg.tags);
        assert_eq!(&msg.tags.0[0].name, "tag1");
        assert_eq!(msg.tags.0[0].value, None);
        assert_eq!(&msg.tags.0[1].name, "tag2");
        assert_eq!(msg.tags.0[1].value, Some("val2".to_string()));
    }
}
