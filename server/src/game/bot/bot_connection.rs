use std::sync::Arc;
use tokio::sync::mpsc;
use crate::packet::ToClientPacket;

/// Channel for sending packets to a bot player
#[derive(Clone, Debug)]
pub struct BotConnection {
    sender: mpsc::UnboundedSender<ToClientPacket>,
    receiver: Arc<std::sync::Mutex<Option<mpsc::UnboundedReceiver<ToClientPacket>>>>,
}

impl BotConnection {
    pub fn new(sender: mpsc::UnboundedSender<ToClientPacket>) -> Self {
        Self { 
            sender,
            receiver: Arc::new(std::sync::Mutex::new(None)),
        }
    }
    
    pub fn new_with_receiver(sender: mpsc::UnboundedSender<ToClientPacket>, receiver: mpsc::UnboundedReceiver<ToClientPacket>) -> Self {
        Self { 
            sender,
            receiver: Arc::new(std::sync::Mutex::new(Some(receiver))),
        }
    }

    pub fn send(&self, packet: ToClientPacket) {
        let _ = self.sender.send(packet);
    }
    
    pub fn take_receiver(&self) -> Option<mpsc::UnboundedReceiver<ToClientPacket>> {
        #[expect(clippy::unwrap_used, reason = "Mutex lock cannot fail, probably")]
        self.receiver.lock().unwrap().take()
    }
}
