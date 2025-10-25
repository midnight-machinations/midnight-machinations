use tokio::sync::mpsc;
use crate::packet::ToClientPacket;

/// Channel for sending packets to a bot player
#[derive(Clone, Debug)]
pub struct BotConnection {
    sender: mpsc::UnboundedSender<ToClientPacket>,
}

impl BotConnection {
    pub fn new(sender: mpsc::UnboundedSender<ToClientPacket>) -> Self {
        Self { sender }
    }

    pub fn send(&self, packet: ToClientPacket) {
        let _ = self.sender.send(packet);
    }
}
