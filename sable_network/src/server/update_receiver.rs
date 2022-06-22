use crate::network::update::*;
use super::*;

impl NetworkUpdateReceiver for Server
{
    fn notify_update(&self, update: NetworkStateChange)
    {
        // This only fails if the receiver has been dropped, which isn't our problem
        self.state_change_sender.send(update).ok();
    }
}