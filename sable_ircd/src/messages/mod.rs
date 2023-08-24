use crate::capability::*;
use sable_network::prelude::*;

#[derive(Debug)]
pub struct UntargetedNumeric {
    caps: CapabilityCondition,
    tags: Vec<OutboundMessageTag>,
    numeric_code: String,
    args: String,
}

impl UntargetedNumeric {
    /// Create a new message
    pub fn new(numeric_code: String, args: String) -> Self {
        Self {
            caps: Default::default(),
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

    /// Specify a set of required client capabilities, without which the formatted message will not be sent
    pub fn with_required_capabilities(mut self, caps: impl Into<ClientCapabilitySet>) -> Self {
        self.caps.require(caps);
        self
    }

    /// Specify a negative capability requirement - the formatted message will only be sent to
    /// clients that do not have this capability
    pub fn except_capability(mut self, caps: impl Into<ClientCapabilitySet>) -> Self {
        self.caps.except(caps);
        self
    }

    /// Provide the source and target information required to convert this to an [`OutboundClientMessage`]
    pub fn format_for(
        self,
        source: &(impl MessageSource + ?Sized),
        target: &(impl MessageTarget + ?Sized),
    ) -> OutboundClientMessage {
        OutboundClientMessage {
            caps: self.caps,
            tags: self.tags,
            content: format!(
                ":{} {} {} {}",
                source.format(),
                self.numeric_code,
                target.format(),
                self.args
            ),
        }
    }

    /// Provide the raw content in a situation where it can't be properly formatted
    pub fn debug_format(&self) -> String {
        format!("* {} * {}", self.numeric_code, self.args)
    }
}

/// A server-to-client protocol message
#[derive(Debug, Clone)]
pub struct OutboundClientMessage {
    caps: CapabilityCondition,
    tags: Vec<OutboundMessageTag>,
    content: String,
}

impl OutboundClientMessage {
    /// Create a new message
    pub fn new(content: String) -> Self {
        Self {
            caps: Default::default(),
            tags: Default::default(),
            content: content,
        }
    }

    /// Add a message tag to the message
    pub fn with_tag(mut self, tag: OutboundMessageTag) -> Self {
        self.tags.push(tag);
        self
    }

    /// Add a set of message tags to the message
    pub fn with_tags(mut self, tags: &Vec<OutboundMessageTag>) -> Self {
        self.tags.extend_from_slice(tags);
        self
    }

    /// Specify a set of required client capabilities, without which the formatted message will not be sent
    pub fn with_required_capabilities(mut self, caps: impl Into<ClientCapabilitySet>) -> Self {
        self.caps.require(caps);
        self
    }

    /// Specify a negative capability requirement - the formatted message will only be sent to
    /// clients that do not have this capability
    pub fn except_capability(mut self, caps: impl Into<ClientCapabilitySet>) -> Self {
        self.caps.except(caps);
        self
    }

    /// Format this message to be sent to a client with the given capabilities
    ///
    /// Returns `None` if the client should not receive the message at all, otherwise
    /// `Some(_)` with the appropriate set of outbound message tags prepended
    pub fn format_for_client_caps(&self, client_caps: ClientCapabilitySet) -> Option<String> {
        // Check capability requirements
        if !self.caps.matches(client_caps) {
            return None;
        }

        let mut result = String::new();

        let mut supported_tags = self.tags.iter().filter(|t| t.caps.matches(client_caps));

        if let Some(first_tag) = supported_tags.next() {
            result.push('@');
            result.push_str(&first_tag.format());

            for tag in supported_tags {
                result.push(';');
                result.push_str(&tag.format());
            }

            result.push(' ');
        }

        result.push_str(&self.content);
        result.push_str("\r\n");

        Some(result)
    }
}

/// A message tag
#[derive(Debug, Clone)]
pub struct OutboundMessageTag {
    name: String,
    value: Option<String>,
    caps: CapabilityCondition,
}

impl OutboundMessageTag {
    /// Construct an outbound message tag, to be attached to a protocol message.
    ///
    /// Unlike in other areas, `caps` is not optional; an outbound message tag
    /// always needs a capability attached to it because legacy clients can't understand
    /// them and there's no single capability that can be relied on to signal a client
    /// that does.
    pub fn new(name: &str, value: Option<String>, caps: impl Into<ClientCapabilitySet>) -> Self {
        Self {
            name: name.to_string(),
            value,
            caps: CapabilityCondition::requires(caps.into()),
        }
    }

    fn format(&self) -> String {
        let mut result = self.name.clone();
        if let Some(value) = &self.value {
            result.push('=');
            result.push_str(value);
        }
        result
    }
}

pub mod batch;
pub mod message;
pub mod numeric;
pub mod send_history;
pub mod send_realtime;

mod message_sink;
pub use message_sink::*;

mod source_target;
pub use source_target::*;
