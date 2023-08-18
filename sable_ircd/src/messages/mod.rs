use sable_network::prelude::*;
use crate::capability::*;

#[derive(Debug)]
pub struct UntargetedNumeric {
    required_caps: ClientCapabilitySet,
    except_caps: ClientCapabilitySet,
    tags: Vec<OutboundMessageTag>,
    numeric_code: String,
    args: String,
}

impl UntargetedNumeric {
    /// Create a new message
    pub fn new(numeric_code: String, args: String) -> Self {
        Self {
            required_caps: Default::default(),
            except_caps: Default::default(),
            tags: Default::default(),
            numeric_code,
            args,
        }
    }

    /// Add a message tag to the numeric
    pub fn with_tag(mut self, tag: OutboundMessageTag) -> Self {
        self.tags.push(tag);
        self
    }

    /// Specify a single required client capability, without which the formatted message will not be sent
    pub fn with_required_capability(mut self, cap: ClientCapability) -> Self {
        self.required_caps.set(cap);
        self
    }

    /// Specify a set of required client capabilities, without which the formatted message will not be sent
    pub fn with_required_capabilities(mut self, caps: ClientCapabilitySet) -> Self {
        self.required_caps.set_all(caps);
        self
    }

    /// Specify a negative capability requirement - the formatted message will only be sent to
    /// clients that do not have this capability
    pub fn except_capability(mut self, caps: ClientCapabilitySet) -> Self {
        self.except_caps = caps;
        self
    }

    /// Provide the source and target information required to convert this to an [`OutboundClientMessage`]
    pub fn format_for(self, source: &(impl MessageSource + ?Sized), target: &(impl MessageTarget + ?Sized)) -> OutboundClientMessage {
        OutboundClientMessage {
            required_caps: self.required_caps,
            except_caps: self.except_caps,
            tags: self.tags,
            content: format!(":{} {} {} {}", source.format(), self.numeric_code, target.format(), self.args)
        }
    }

    /// Provide the raw content in a situation where it can't be properly formatted
    pub fn debug_format(&self) -> String {
        format!("* {} * {}", self.numeric_code, self.args)
    }
}

/// A server-to-client protocol message
#[derive(Debug)]
pub struct OutboundClientMessage {
    required_caps: ClientCapabilitySet,
    except_caps: ClientCapabilitySet,
    tags: Vec<OutboundMessageTag>,
    content: String,
}

impl OutboundClientMessage {
    /// Create a new message
    pub fn new(content: String) -> Self {
        Self {
            required_caps: Default::default(),
            except_caps: Default::default(),
            tags: Default::default(),
            content: content
        }
    }

    /// Add a message tag to the message
    pub fn with_tag(mut self, tag: OutboundMessageTag) -> Self {
        self.tags.push(tag);
        self
    }

    /// Specify a single required client capability, without which the formatted message will not be sent
    pub fn with_required_capability(mut self, cap: ClientCapability) -> Self {
        self.required_caps.set(cap);
        self
    }

    /// Specify a required client capability, without which this message will not be sent
    pub fn with_required_capabilities(mut self, caps: ClientCapabilitySet) -> Self {
        self.required_caps = caps;
        self
    }

    /// Specify a negative capability requirement - this message will only be sent to
    /// clients that do not have this capability
    pub fn except_capability(mut self, caps: ClientCapabilitySet) -> Self {
        self.except_caps = caps;
        self
    }

    /// Format this message to be sent to a client with the given capabilities
    ///
    /// Returns `None` if the client should not receive the message at all, otherwise
    /// `Some(_)` with the appropriate set of outbound message tags prepended
    pub fn format_for_client_caps(&self, caps: &ClientCapabilitySet) -> Option<String> {
        // If the target connection doesn't support all the message's required caps,
        // don't send it
        if ! caps.has_all(self.required_caps) {
            return None;
        }

        // If the target does have any of our except capabilities, don't send it
        if caps.has_any(self.except_caps) {
            return None;
        }

        let mut result = String::new();

        // If the target connection doesn't support message tags, don't send them
        if caps.has(ClientCapability::MessageTags) {
            let mut supported_tags = self.tags.iter().filter(|t| caps.has(t.required_cap));

            if let Some(first_tag) = supported_tags.next()
            {
                result.push('@');
                result.push_str(&first_tag.format());

                for tag in supported_tags
                {
                    result.push(';');
                    result.push_str(&tag.format());
                }

                result.push(' ');
            }
        }

        result.push_str(&self.content);
        result.push_str("\r\n");

        Some(result)
    }
}

/// A message tag
#[derive(Debug,Clone)]
pub struct OutboundMessageTag
{
    name: String,
    value: Option<String>,
    required_cap: ClientCapability,
}

impl OutboundMessageTag
{
    pub fn new(name: &str, value: Option<String>, required_cap: ClientCapability) -> Self
    {
        Self { name: name.to_string(), value , required_cap }
    }

    fn format(&self) -> String
    {
        let mut result = self.name.clone();
        if let Some(value) = &self.value {
            result.push('=');
            result.push_str(value);
        }
        result
    }
}

pub mod message;
pub mod numeric;
pub mod send_history;
pub mod send_realtime;

mod message_sink;
pub use message_sink::*;

mod source_target;
pub use source_target::*;