use std::collections::VecDeque;
use std::time::Instant;

use serde::Serialize;

use crate::game::Game;
use crate::game::game_client::{GameClient, GameClientLocation};
use crate::{client_connection::ClientConnection, packet::ToClientPacket, websocket_connections::connection::ClientSender};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LobbyClient{
    pub connection: ClientConnection,
    pub ready: Ready,
    pub client_type: LobbyClientType,
    
    #[serde(skip)]
    pub last_message_times: VecDeque<Instant>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Ready {
    Host,
    Ready,
    NotReady,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum LobbyClientType{
    Spectator{
        name: String,
    },
    Player{
        name: String,
    }
}

impl LobbyClient {
    pub fn new(name: String, connection: ClientSender, host: bool)->Self{
        LobbyClient{
            connection: ClientConnection::Connected(connection),
            ready: if host { Ready::Host } else { Ready::NotReady },
            client_type: LobbyClientType::Player{name},
            last_message_times: VecDeque::new()
        }
    }
    pub fn new_from_game_client(game: &Game, game_client: GameClient)->Self{

        match game_client.client_location {
            GameClientLocation::Player(player) => {
                LobbyClient{
                    connection: player.connection(game).clone(),
                    ready: if game_client.host { Ready::Host } else { Ready::NotReady },
                    client_type: LobbyClientType::Player{name: player.name(game).to_string()},
                    last_message_times: VecDeque::new()
                }
            },
            GameClientLocation::Spectator(spectator) => {
                LobbyClient{
                    connection: spectator.connection(game),
                    ready: if game_client.host { Ready::Host } else { Ready::Ready },
                    client_type: LobbyClientType::Spectator{
                        name: spectator.name(game).unwrap_or_else(|| "Spectator".to_string())
                    },
                    last_message_times: VecDeque::new()
                }
            }
        }

        
    }
    pub fn set_host(&mut self) {
        self.ready = Ready::Host;
    }
    pub fn relinquish_host(&mut self) {
        self.ready = Ready::NotReady;
    }

    pub fn is_host(&self) -> bool {
        self.ready == Ready::Host
    }

    pub fn is_spectator(&self) -> bool {
        matches!(self.client_type, LobbyClientType::Spectator { .. })
    }

    pub fn send(&self, message: ToClientPacket) {
        if let ClientConnection::Connected(ref sender) = self.connection {
            sender.send(message);
        }
    }
}