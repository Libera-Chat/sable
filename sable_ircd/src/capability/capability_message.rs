use crate::messages::MessageTypeFormat;
use crate::capability::{
    ClientCapability,
    ClientCapabilitySet,
};

#[derive(Debug,Clone)]
pub struct CapabilityMessage<T>
{
    message: T,
    required_caps: ClientCapabilitySet,
}

impl<T: MessageTypeFormat> MessageTypeFormat for CapabilityMessage<T>
{
    fn format_for_client_caps(&self, caps: &ClientCapabilitySet) -> Option<String>
    {
        if caps.has_all(self.required_caps)
        {
            self.message.format_for_client_caps(caps)
        }
        else
        {
            None
        }
    }
}

pub trait CapableMessage : MessageTypeFormat + Sized
{
    fn with_required_capabilities(self, caps: ClientCapabilitySet) -> CapabilityMessage<Self>;

    fn with_required_capability(self, cap: ClientCapability) -> CapabilityMessage<Self>
    {
        self.with_required_capabilities(cap.into())
    }
}

impl<T: MessageTypeFormat + Sized> CapableMessage for T
{
    fn with_required_capabilities(self, caps: ClientCapabilitySet) -> CapabilityMessage<Self>
    {
        CapabilityMessage { message: self, required_caps: caps }
    }
}