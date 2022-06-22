use super::*;
use crate::messages::*;

/// A message tag
#[derive(Debug,Clone)]
pub struct MessageTag
{
    pub name: String,
    pub value: String,
    pub required_cap: ClientCapability,
}

impl MessageTag
{
    pub fn new(name: &str, value: String, required_cap: ClientCapability) -> Self
    {
        Self { name: name.to_string(), value , required_cap }
    }
}

impl std::fmt::Display for MessageTag
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        f.write_str(&self.name)?;
        f.write_str("=")?;
        f.write_str(&self.value)?;
        Ok(())
    }
}

#[derive(Debug,Clone)]
pub struct TaggedMessage<T>
{
    message: T,
    tags: Vec<MessageTag>
}

impl <'a, T: MessageType> MessageTypeFormat for TaggedMessage<T>
{
    fn format_for_client_caps(&self, caps: &ClientCapabilitySet) -> String
    {
        let mut result = String::new();

        let mut supported_tags = self.tags.iter().filter(|t| caps.has(t.required_cap));

        if let Some(first_tag) = supported_tags.next()
        {
            result.push_str("@");
            result.push_str(&first_tag.name);
            result.push_str("=");
            result.push_str(&first_tag.value);

            for tag in supported_tags
            {
                result.push_str(";");
                result.push_str(&tag.name);
                result.push_str("=");
                result.push_str(&tag.value);
            }

            result.push_str(" ");
        }

        result.push_str(&self.message.format_for_client_caps(caps));

        result
    }
}

pub trait TaggableMessage : Sized
{
    type Tagged;
    fn with_tags(self, tags: Vec<MessageTag>) -> Self::Tagged;
    fn with_tag(self, tag: MessageTag) -> Self::Tagged;
}

impl<T: MessageType + Sized> TaggableMessage for T
{
    type Tagged = TaggedMessage<T>;

    fn with_tags(self, tags: Vec<MessageTag>) -> TaggedMessage<T>
    {
        TaggedMessage { message: self, tags }
    }

    fn with_tag(self, tag: MessageTag) -> TaggedMessage<T>
    {
        TaggedMessage {
            message: self,
            tags: vec!(tag)
        }
    }
}

impl<T: MessageType + Sized> TaggableMessage for TaggedMessage<T>
{
    type Tagged = Self;

    fn with_tags(mut self, tags: Vec<MessageTag>) -> Self::Tagged
    {
        self.tags.extend(tags);
        self
    }

    fn with_tag(mut self, tag: MessageTag) -> Self::Tagged
    {
        self.tags.push(tag);
        self
    }
}