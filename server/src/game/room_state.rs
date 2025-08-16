use std::time::Duration;
use super::event::on_tick::OnTick;
use crate::client_connection::ClientConnection;
use crate::game::chat::ChatGroup;
use crate::game::chat::ChatMessageVariant;
use crate::game::components::synopsis::SynopsisTracker;
use crate::game::event::on_game_ending::OnGameEnding;
use crate::game::game_client::GameClient;
use crate::game::game_client::GameClientLocation;
use crate::game::game_conclusion::GameConclusion;
use crate::game::phase::PhaseStateMachine;
use crate::game::player::PlayerReference;
use crate::game::spectator::spectator_pointer::SpectatorPointer;
use crate::game::spectator::SpectatorInitializeParameters;
use crate::game::Game;
use crate::game::GameOverReason;
use crate::packet::RejectJoinReason;
use crate::packet::RoomPreviewData;
use crate::packet::ToClientPacket;
use crate::room::JoinRoomClientResult;
use crate::room::RemoveRoomClientResult;
use crate::room::RoomClientID;
use crate::room::RoomState;
use crate::room::RoomTickResult;
use crate::websocket_connections::connection::ClientSender;

impl RoomState for Game {
    fn tick(&mut self, time_passed: Duration) -> RoomTickResult {
        if !self.ticking { 
            return RoomTickResult { close_room: false }
        }

        if let Some(conclusion) = GameConclusion::game_is_over(self) {
            OnGameEnding::new(conclusion).invoke(self);
        }

        if self.phase_machine.day_number == u8::MAX {
            self.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::GameOver { 
                synopsis: SynopsisTracker::get(self, GameConclusion::Draw)
            });
            self.send_packet_to_all(ToClientPacket::GameOver{ reason: GameOverReason::ReachedMaxDay });
            self.ticking = false;
            return RoomTickResult { close_room: !self.is_any_client_connected() };
        }

        while self.phase_machine.time_remaining.is_some_and(|d| d.is_zero()) {
            PhaseStateMachine::next_phase(self, None);
        }
        PlayerReference::all_players(self).for_each(|p|p.tick(self, time_passed));
        SpectatorPointer::all_spectators(self).for_each(|s|s.tick(self, time_passed));

        self.phase_machine.time_remaining = self.phase_machine.time_remaining.map(|d|d.saturating_sub(time_passed));

        OnTick::new().invoke(self);

        RoomTickResult {
            close_room: !self.is_any_client_connected()
        }
    }
    
    fn send_to_client_by_id(&self, room_client_id: RoomClientID, packet: ToClientPacket) {
        if let Some(player) = self.clients.get(&room_client_id) {
            match player.client_location {
                GameClientLocation::Player(player) => player.send_packet(self, packet),
                GameClientLocation::Spectator(spectator) => spectator.send_packet(self, packet)
            }
        }
    }
    
    fn join_client(&mut self, send: &ClientSender) -> Result<JoinRoomClientResult, RejectJoinReason> {
        let is_host = !self.clients.iter().any(|p|p.1.host);
                
        let Some(room_client_id) = 
            (self.clients
                .iter()
                .map(|(i,_)|*i)
                .fold(0u32, u32::max) as RoomClientID).checked_add(1) else {
                    return Err(RejectJoinReason::RoomFull);
                };

        self.ensure_host_exists(None);

        let new_spectator = self.join_spectator(SpectatorInitializeParameters {
            connection: ClientConnection::Connected(send.clone()),
            host: is_host,
        })?;
        
        let new_client = GameClient::new_spectator(new_spectator, is_host);

        self.clients.insert(room_client_id, new_client);

        self.resend_host_data_to_all_hosts();
        Ok(JoinRoomClientResult { id: room_client_id, in_game: true, spectator: true })
    }

    fn initialize_client(&mut self, room_client_id: RoomClientID, send: &ClientSender) {
        if let Some(client) = self.clients.get(&room_client_id) {
            match client.client_location {
                GameClientLocation::Player(player) => {
                    player.connect(self, send.clone());
                    player.send_join_game_data(self);
                },
                GameClientLocation::Spectator(spectator) => {
                    spectator.send_join_game_data(self);
                }
            }
        }
        
        send.send(ToClientPacket::PlayersHost{hosts:
            self.clients
                .iter()
                .filter(|p|p.1.host)
                .map(|p|*p.0)
                .collect()
        });

        send.send(ToClientPacket::RoomName { name: self.room_name.clone() });
    }
    
    fn remove_client(&mut self, room_client_id: u32) -> RemoveRoomClientResult {
        let Some(game_player) = self.clients.get_mut(&room_client_id) else {
            return RemoveRoomClientResult::ClientNotInRoom;
        };

        match game_player.client_location {
            GameClientLocation::Player(player) => player.quit(self),
            GameClientLocation::Spectator(spectator) => {
                self.clients.remove(&room_client_id);

                // Shift every other spectator down one index
                for client in self.clients.iter_mut() {
                    if let GameClientLocation::Spectator(other) = &mut client.1.client_location {
                        if other.index() > spectator.index() {
                            *other = SpectatorPointer::new(other.index().saturating_sub(1));
                        }
                    }
                }

                self.remove_spectator(spectator.index());
            }
        }

        self.ensure_host_exists(None);

        self.resend_host_data_to_all_hosts();

        if !self.is_any_client_connected() {
            RemoveRoomClientResult::RoomShouldClose
        } else {
            RemoveRoomClientResult::Success
        }
    }
    
    fn remove_client_rejoinable(&mut self, id: u32) -> RemoveRoomClientResult {
        let Some(game_player) = self.clients.get_mut(&id) else { return RemoveRoomClientResult::ClientNotInRoom };

        match game_player.client_location {
            GameClientLocation::Player(player) => {
                if !player.is_disconnected(self) {
                    player.lose_connection(self);
    
                    self.ensure_host_exists(None);
                    self.resend_host_data_to_all_hosts();
                }
            },
            GameClientLocation::Spectator(spectator) => {
                self.clients.remove(&id);

                // Shift every other spectator down one index
                for client in self.clients.iter_mut() {
                    if let GameClientLocation::Spectator(other) = &mut client.1.client_location {
                        if other.index() > spectator.index() {
                            *other = SpectatorPointer::new(other.index().saturating_sub(1));
                        }
                    }
                }

                self.remove_spectator(spectator.index());
            }
        }

        RemoveRoomClientResult::Success
    }
    
    fn rejoin_client(&mut self, _: &ClientSender, room_client_id: u32) -> Result<JoinRoomClientResult, RejectJoinReason> {
        let Some(client) = self.clients.get_mut(&room_client_id) else {
            return Err(RejectJoinReason::PlayerDoesntExist)
        };
        
        if let GameClientLocation::Player(player) = client.client_location {
            if !player.could_reconnect(self) {
                return Err(RejectJoinReason::PlayerTaken)
            };

            self.resend_host_data_to_all_hosts();

            Ok(JoinRoomClientResult { id: room_client_id, in_game: true, spectator: false })
        }else{
            Err(RejectJoinReason::PlayerDoesntExist)
        }
    }
    
    fn get_preview_data(&self) -> RoomPreviewData {
        RoomPreviewData {
            name: self.room_name.clone(),
            in_game: true,
            players: self.clients.iter()
                .filter_map(|(id, player)|
                    if let GameClientLocation::Player(player) = player.client_location {
                        Some((*id, player.name(self).clone()))
                    } else {
                        None
                    }
                )
                .collect()
        }
    }
    
    fn is_host(&self, room_client_id: u32) -> bool {
        if let Some(client) = self.clients.get(&room_client_id){
            client.host
        }else{
            false
        }
    }
}

