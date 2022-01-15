use crate::*;
use crate::event::*;
use crate::update::*;

impl Network
{
    pub(super) fn load_config(&mut self, _target: ConfigId, _event: &Event, details: &details::LoadConfig, _updates: &dyn NetworkUpdateReceiver)
    {
        self.config = details.config.clone();
    }
}