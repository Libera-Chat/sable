use super::*;

#[derive(Debug,Default,Clone,Copy)]
pub struct CapabilityCondition {
    must_have: ClientCapabilitySet,
    must_not_have: ClientCapabilitySet,
}

impl CapabilityCondition {
    /// Construct a new condition set that requires the given capabilities and has no
    /// negative constraint
    pub fn requires(caps: impl Into<ClientCapabilitySet>) -> Self {
        Self {
            must_have: caps.into(),
            must_not_have: ClientCapabilitySet::new()
        }
    }

    /// Add a required capability to the condition set
    pub fn require(&mut self, caps: impl Into<ClientCapabilitySet>) {
        self.must_have.set_all(caps.into())
    }

    /// Add an except capability, which will cause the condition not to match if it is set
    pub fn except(&mut self, caps: impl Into<ClientCapabilitySet>) {
        self.must_not_have.set_all(caps.into())
    }

    /// Determine whether the given set of client capabilities matches this condition.
    ///
    /// Returns true iff all required flags, and no except flags, are present in `caps`
    pub fn matches(&self, caps: ClientCapabilitySet) -> bool {
        caps.has_all(self.must_have) && ! caps.has_any(self.must_not_have)
    }
}