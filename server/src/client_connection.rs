use std::time::Duration;

use serde::Serialize;

use crate::{game::bot::BotConnection, packet::ToClientPacket, websocket_connections::connection::ClientSender};

#[derive(Clone, Debug)]
pub enum ClientConnection {
    Connected(ClientSender),
    Bot(BotConnection),
    CouldReconnect { disconnect_timer: Duration },
    Disconnected
}
impl ClientConnection {
    pub fn send_packet(&self, packet: ToClientPacket)->bool {
        match self {
            ClientConnection::Connected(sender) => {
                sender.send(packet);
                true
            }
            ClientConnection::Bot(bot_conn) => {
                bot_conn.send(packet);
                true
            }
            _ => false
        }
    }
}
impl Serialize for ClientConnection{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        match self {
            ClientConnection::Connected(_) => serializer.serialize_str("connected"),
            ClientConnection::Bot(_) => serializer.serialize_str("bot"),
            ClientConnection::CouldReconnect { .. } => {serializer.serialize_str("couldReconnect")}
            ClientConnection::Disconnected => serializer.serialize_str("disconnected"),
        }
    }
}