mod player_accessors;
mod player_reference;
mod player_send_packet;
mod player_tick;
mod player_helper_functions;
mod player_event_listeners;

pub use player_reference::PlayerIndex;
pub use player_reference::PlayerReference;

use crate::{
    client_connection::ClientConnection,
    websocket_connections::connection::ClientSender,
};

pub struct PlayerInitializeParameters {
    pub connection: ClientConnection,
    pub name: String,
    pub host: bool,
}
pub struct Player {
    connection: ClientConnection,

    name: String,
    alive: bool,
    notes: Vec<String>,
    crossed_out_outlines: Vec<u8>,
    death_note: Option<String>,
}
impl Player {
    pub fn new(name: String, sender: ClientSender) -> Self {
        Self {
            connection: ClientConnection::Connected(sender),

            name,
            alive: true,
            notes: vec![],
            crossed_out_outlines: vec![],
            death_note: None,
        }
    }
}

pub mod test {
    use crate::client_connection::ClientConnection;

    use super::Player;

    pub fn mock_player(name: String) -> Player {
        Player {
            connection: ClientConnection::CouldReconnect { disconnect_timer: None },

            name,
            alive: true,
            notes: vec![],
            crossed_out_outlines: vec![],
            death_note: None,
        }
    }
}
