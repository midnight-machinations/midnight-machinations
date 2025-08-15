use std::time::Duration;

use serde::Serialize;

use crate::{packet::ToClientPacket, websocket_connections::connection::ClientSender};

#[derive(Clone, Debug)]
pub enum ClientConnection {
    Connected(ClientSender),
    CouldReconnect { disconnect_timer: Duration },
    Disconnected
}
impl ClientConnection {
    pub fn send_packet(&self, packet: ToClientPacket)->bool {
        if let ClientConnection::Connected(sender) = self {
            sender.send(packet);
            true
        }else{
            false
        }
    }
}
impl Serialize for ClientConnection{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        match self {
            ClientConnection::Connected(_) => serializer.serialize_str("connected"),
            ClientConnection::CouldReconnect { .. } => {serializer.serialize_str("couldReconnect")}
            ClientConnection::Disconnected => serializer.serialize_str("disconnected"),
        }
    }
}