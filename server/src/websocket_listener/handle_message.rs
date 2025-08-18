use std::collections::HashMap;

use crate::{game::on_client_message::GameClientMessageResult, lobby::on_client_message::LobbyClientMessageResult, log, packet::{RoomPreviewData, RejectJoinReason, ToClientPacket, ToServerPacket}, room::{on_client_message::RoomClientMessageResult, RemoveRoomClientResult, Room, RoomState}};

use super::{client::{ClientLocation, ClientReference}, RoomCode, WebsocketListener};

impl WebsocketListener{
    pub(super) fn handle_message(&mut self, client: ClientReference, packet: ToServerPacket) {

        match packet {
            ToServerPacket::Ping => {
                client.deref_mut(self).on_ping();
            },
            ToServerPacket::RoomListRequest => {
                client.send(
                    self,
                    ToClientPacket::RoomList{rooms: self.rooms()
                        .iter()
                        .map(|(room_code, room)| (*room_code, room.get_preview_data()))
                        .collect::<HashMap<RoomCode, RoomPreviewData>>()
                    }
                );
            },
            ToServerPacket::ReJoin {room_code, player_id } => {
                self.set_client_in_room_reconnect(client, room_code, player_id);
            }
            ToServerPacket::Join{ room_code } => {
                self.set_client_in_room(&client, room_code);
            },
            ToServerPacket::Host => {
                let Some(room_code) = self.create_room() else {
                    client.deref(self).send(ToClientPacket::RejectJoin { reason: RejectJoinReason::ServerBusy });
                    return;
                };
                
                self.set_client_in_room(&client, room_code);

                log!(important "Room"; "Created {room_code}");
            },
            ToServerPacket::Leave => {
                self.set_client_outside_room(&client, false);
            },
            ToServerPacket::Kick { player_id: kicked_player_id } => {
                let Ok((room,room_code,host_id)) = client.get_room_mut(self) else {return};

                if !room.is_host(host_id) {return}

                
                let kicked_player_client_ref = 
                    ClientReference::all_clients(self)
                    .find(|c|
                        *c.location(self) == ClientLocation::InRoom { room_code, room_client_id: kicked_player_id }
                    );

                if let Some(kicked_player_client_ref) = kicked_player_client_ref {

                    kicked_player_client_ref.send(self, ToClientPacket::RejectJoin { reason: RejectJoinReason::ServerBusy });
                    self.set_client_outside_room(&kicked_player_client_ref, false);
                    
                } else { // The client wasn't in the Websocket Listener
                    let Ok((room,_,_)) = client.get_room_mut(self) else {return};

                    match room.remove_client(kicked_player_id) {
                        RemoveRoomClientResult::Success => {}
                        RemoveRoomClientResult::RoomShouldClose => self.delete_room(room_code),
                        RemoveRoomClientResult::ClientNotInRoom => {
                            // The client is already not in the listener, so doing nothing is fine
                        }
                    }
                }
                
            },
            _ => {
                let sender = &client.sender(self);
                let Ok((room, room_code, id)) = client.get_room_mut(self) else {return};

                match room.on_client_message(sender, id, packet) {
                    RoomClientMessageResult::LobbyAction(LobbyClientMessageResult::StartGame(game)) => {
                        log!(info "Room"; "Game started with room code {}", room_code);

                        *room = Room::Game(*game);
                    },
                    RoomClientMessageResult::GameAction(GameClientMessageResult::BackToLobby(lobby)) => {
                        *room = Room::Lobby(lobby);
                    },
                    RoomClientMessageResult::GameAction(GameClientMessageResult::Close) |
                    RoomClientMessageResult::LobbyAction(LobbyClientMessageResult::Close) => {
                        self.delete_room(room_code);
                    },
                    _ => {}
                }
                
            }
        }
    }
}