use std::collections::VecDeque;
use crate::platform::Instant;

use serde::Serialize;

use super::{player::PlayerReference, spectator::spectator_pointer::SpectatorPointer};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameClient{
    #[serde(rename = "clientType")]
    pub client_location: GameClientLocation,
    pub host: bool,

    #[serde(skip)]
    pub last_message_times: VecDeque<Instant>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content = "index", rename_all="camelCase")]
pub enum GameClientLocation {
    Player(PlayerReference),
    Spectator(SpectatorPointer)
}
impl GameClient {
    pub fn new_spectator(pointer: SpectatorPointer, host: bool)->Self{
        GameClient{
            client_location: GameClientLocation::Spectator(pointer),
            host,
            last_message_times: VecDeque::new(),
        }
    }
    pub fn set_host(&mut self) {
        self.host = true;
    }
    pub fn relinquish_host(&mut self) {
        self.host = false;
    }
}