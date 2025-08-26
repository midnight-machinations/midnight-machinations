use std::time::Duration;
use tokio_tungstenite::tungstenite::Message;

use crate::{game::on_client_message::GameClientMessageResult, log, packet::ToServerPacket, room::{Room, RoomState}, websocket_connections::connection::Connection};

use super::{client::ClientReference, WebsocketListener, ValidateClientError};

impl WebsocketListener{
    pub fn on_connect(&mut self, connection: &Connection) {
        self.create_client(connection);
    }

    pub fn on_disconnect(&mut self, connection: Connection) {
        if let Some(client) = ClientReference::new(connection.address(), self){
            self.delete_client(&client);
        }
    }

    pub fn on_message(&mut self, connection: &Connection, message: &Message) {
        if message.is_empty() { return }

        log!(info "Listener"; "{}: {}", &connection.address().to_string(), message);

        let Ok(packet) = serde_json::from_str::<ToServerPacket>(message.to_string().as_str()) else {
            log!(error "Listener"; "Received message but could not parse packet");
            return
        };

        match self.validate_client(connection.address()) {
            Err(ValidateClientError::ClientDoesntExist) =>
                log!(error "Listener"; "Received packet from an address with no client"),
            Err(ValidateClientError::InRoomThatDoesntExist) => 
                log!(error "Listener"; "Received packet from a client in a room that doesnt exist"),
            Ok(client) => {
                self.handle_message(client, packet)
            }
        }
    }
    
    pub(super) fn tick(&mut self, delta_time: Duration){
        let mut closed_rooms = Vec::new();
        let mut closed_clients = Vec::new();
        let mut sent_to_lobby = Vec::new();

        for (room_code, room) in self.rooms_mut().iter_mut() {
            let tick_data = room.tick(delta_time);
            if tick_data.close_room {
                closed_rooms.push(*room_code);
            } else if tick_data.send_to_lobby {
                'to_lobby: { match &mut **room {
                    Room::Game(game) => {
                        //creating the lobby now not later to prevent borrow errors
                        let GameClientMessageResult::BackToLobby(lobby) = game.back_to_lobby() else {
                            log!(error "Listener"; "Game.back_to_lobby() did not return a GameClientMessageResult::BackToLobby. Room code: {room_code}");
                            break 'to_lobby;
                        };
                        sent_to_lobby.push((*room_code, lobby));
                    },

                    Room::Lobby(_) => {
                        log!(error "Listener"; "Lobby tried to return to lobby. Room code: {:?}", room_code);
                    },
                }}
            }
        }

        for (room_code, lobby) in sent_to_lobby {
            if let Some(room) = self.get_room_mut(&room_code) {
                *room = Room::Lobby(lobby);
                log!(info "Listener"; "Room {room_code} forcibly moved to lobby.");
            } else {
                log!(error "Listener"; "Room {room_code} was supposed to be moved from game to lobby but does not exist.");
            }
        }

        for client in ClientReference::all_clients(self){
            client.tick(self);
            if client.ping_timed_out(self) {
                closed_clients.push(client);
            }
        }

        for room_code in closed_rooms {
            self.delete_room(room_code);
        }
        for client in closed_clients {
            log!(important "Listener"; "Closed connection {} due to ping timed out", client.address(self));
            self.delete_client(&client);
        }
    }

}