mod player_accessors;
mod player_reference;
mod player_send_packet;
mod player_tick;
mod player_helper_functions;
mod player_event_listeners;

pub use player_reference::PlayerIndex;
pub use player_reference::PlayerReference;

use crate::{
    vec_set::VecSet,
    client_connection::ClientConnection,
    game::role::{Role, RoleState},
    game::ability::PlayerAbilities,
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
    role_state: RoleState,
    abilities: PlayerAbilities,
    alive: bool,
    notes: Vec<String>,
    crossed_out_outlines: Vec<u8>,
    death_note: Option<String>,

    role_labels: VecSet<PlayerReference>,
}
impl Player {
    pub fn new(name: String, sender: ClientSender, role: Role) -> Self {
        Self {
            connection: ClientConnection::Connected(sender),

            name,
            role_state: role.default_state(),
            abilities: PlayerAbilities::new(),
            alive: true,
            notes: vec![],
            crossed_out_outlines: vec![],
            death_note: None,

            role_labels: VecSet::new(),
        }
    }
}

pub mod test {
    use std::time::Duration;

    use crate::{client_connection::ClientConnection, game::role::Role, vec_set::VecSet, game::ability::PlayerAbilities};

    use super::Player;

    pub fn mock_player(name: String, role: Role) -> Player {
        Player {
            // Since `tick` is never called in tests, this will never decrement.
            connection: ClientConnection::CouldReconnect { disconnect_timer: Duration::from_secs(1) },

            name,
            role_state: role.default_state(),
            abilities: PlayerAbilities::new(),
            alive: true,
            notes: vec![],
            crossed_out_outlines: vec![],
            death_note: None,

            role_labels: VecSet::new(),
        }
    }
}
