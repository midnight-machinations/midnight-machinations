mod player_accessors;
mod player_reference;
mod player_send_packet;
mod player_reset;
mod player_helper_functions;
mod player_event_listeners;

pub use player_reference::PlayerIndex;
pub use player_reference::PlayerReference;

use crate::client_connection::ClientConnection;
use crate::vec_set::VecSet;
use crate::{
    game::{
        role::{Role, RoleState}, 
        verdict::Verdict,
    },
    websocket_connections::connection::ClientSender,
};
use super::chat::ChatMessage;

pub struct PlayerInitializeParameters {
    pub connection: ClientConnection,
    pub name: String,
    pub host: bool,
}
pub struct Player {
    connection: ClientConnection,

    name: String,
    role_state: RoleState,
    alive: bool,
    notes: Vec<String>,
    crossed_out_outlines: Vec<u8>,
    death_note: Option<String>,

    role_labels: VecSet<PlayerReference>,

    chat_messages: Vec<ChatMessage>,
    queued_chat_messages: Vec<ChatMessage>, // Not yet sent to the client

    fast_forward_vote: bool,

    voting_variables: PlayerVotingVariables,
}
struct PlayerVotingVariables{
    verdict:        Verdict,
}
impl Player {
    pub fn new(name: String, sender: ClientSender, role: Role) -> Self {
        Self {
            connection: ClientConnection::Connected(sender),

            name,
            role_state: role.default_state(),
            alive: true,
            notes: vec![],
            crossed_out_outlines: vec![],
            death_note: None,

            role_labels: VecSet::new(),

            chat_messages: Vec::new(),
            queued_chat_messages: Vec::new(),
            
            fast_forward_vote: false,

            voting_variables: PlayerVotingVariables{
                verdict : Verdict::Abstain,
            },
        }
    }
}

pub mod test {
    use std::time::Duration;

    use crate::{client_connection::ClientConnection, game::{role::Role, verdict::Verdict}, vec_set::VecSet};

    use super::{Player, PlayerVotingVariables};

    pub fn mock_player(name: String, role: Role) -> Player {
        Player {
            // Since `tick` is never called in tests, this will never decrement.
            connection: ClientConnection::CouldReconnect { disconnect_timer: Duration::from_secs(1) },

            name,
            role_state: role.default_state(),
            alive: true,
            notes: vec![],
            crossed_out_outlines: vec![],
            death_note: None,

            role_labels: VecSet::new(),

            chat_messages: Vec::new(),
            queued_chat_messages: Vec::new(),

            fast_forward_vote: false,

            voting_variables: PlayerVotingVariables{
                verdict : Verdict::Abstain,
            },
        }
    }
}
