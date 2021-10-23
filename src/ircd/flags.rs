use bitflags::bitflags;

bitflags! {
    #[derive(serde::Serialize,serde::Deserialize)]
    pub struct ChannelModeFlags: u32 {
        const NO_EXTERNAL = 0x01;
    }
}

impl Default for ChannelModeFlags
{
    fn default() -> Self
    {
        Self::NO_EXTERNAL
    }
}

