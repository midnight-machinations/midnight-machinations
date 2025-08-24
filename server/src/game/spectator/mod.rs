pub mod spectator_pointer;

use std::collections::VecDeque;

use crate::{
    client_connection::ClientConnection, game::chat::{ChatMessageIndex, ChatMessageVariant},
    packet::ToClientPacket
};

#[derive(Debug, Clone)]
pub struct SpectatorInitializeParameters {
    pub connection: ClientConnection,
    pub host: bool,
}
pub struct Spectator {
    pub connection: ClientConnection,
    pub fast_forward_vote: bool,

    pub queued_chat_messages: VecDeque<(ChatMessageIndex, ChatMessageVariant)>,
}
impl Spectator {
    pub fn new(params: SpectatorInitializeParameters) -> Self {
        Self {
            connection: params.connection,
            fast_forward_vote: false,

            queued_chat_messages: VecDeque::new(),
        }
    }
    pub fn send_packet(&self, packet: ToClientPacket) {
        self.connection.send_packet(packet);
    }
    pub fn send_packets(&self, packets: Vec<ToClientPacket>) {
        for packet in packets {
            self.send_packet(packet);
        }
    }
}