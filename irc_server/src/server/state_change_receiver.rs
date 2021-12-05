use super::*;

pub(super) struct StateChangeReceiver
{
    pub send: std::sync::mpsc::Sender<NetworkStateChange>,
    pub recv: std::sync::mpsc::Receiver<NetworkStateChange>
}

impl StateChangeReceiver
{
    pub fn new() -> Self
    {
        let (send, recv) = std::sync::mpsc::channel();
        Self {
            send: send,
            recv: recv,
        }
    }
}

impl NetworkUpdateReceiver for StateChangeReceiver
{
    fn notify_update(&self, update: NetworkStateChange)
    {
        self.send.send(update).expect("Failed to transmit network state change");
    }
}