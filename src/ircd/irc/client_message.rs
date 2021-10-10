use crate::ircd::*;

pub struct ClientMessage
{
    pub source: Id,
    pub command: String,
    pub args: Vec<String>
}

impl ClientMessage
{
    pub fn parse(source: Id, raw: &str) -> Option<Self>
    {
        let mut args = Vec::new();
        let offset = match raw.find(" ") {
            Some(offset) => offset,
            None => {
                return Some(Self {
                    source: source,
                    command: raw.to_string(),
                    args: Vec::new()
                });
            }
        };
        let command = &raw[0 .. offset];
        let mut rest = &raw[offset+1 .. ];

        loop {
            if rest.starts_with(":") {
                let arg = &rest[1..];
                if !arg.is_empty() { 
                    args.push(arg.to_string());
                }
                break;
            }
            match rest.find(" ") {
                Some(offset) => {
                    let arg = &rest[0 .. offset];
                    if !arg.is_empty() {
                        args.push(arg.to_string());
                    }
                    rest = &rest[offset + 1 ..];
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
            source: source,
            command: command.to_string(),
            args: args
        })
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn no_args()
    {
        let msg = ClientMessage::parse(Id::new(0,0), "command").unwrap();
        assert_eq!(msg.command, "command");
        assert_eq!(msg.args.len(), 0);
    }

    #[test]
    fn simple_args()
    {
        let msg = ClientMessage::parse(Id::new(0,0), "command arg1 arg2 :arg three").unwrap();
        
        assert_eq!(msg.command, "command");
        assert_eq!(msg.args, &["arg1", "arg2", "arg three"]);
    }

    #[test]
    fn ending_space()
    {
        let msg = ClientMessage::parse(Id::new(0,0), "command arg1 arg2 ").unwrap();

        assert_eq!(msg.args, &["arg1", "arg2"]);
    }

    #[test]
    fn ending_colon()
    {
        let msg = ClientMessage::parse(Id::new(0,0), "command arg1 arg2 :").unwrap();

        assert_eq!(msg.args, &["arg1", "arg2"]);

    }

    #[test]
    fn double_space()
    {
        let msg = ClientMessage::parse(Id::new(0,0), "command arg1  arg2").unwrap();

        assert_eq!(msg.args, &["arg1", "arg2"]);
    }

    #[test]
    fn colon_space()
    {
        let msg = ClientMessage::parse(Id::new(0,0), "command arg1 : arg2").unwrap();

        assert_eq!(msg.args, &["arg1", " arg2"]);
    }
}