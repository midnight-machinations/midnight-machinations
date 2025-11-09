use crate::{
    game::{
        abilities::role_abilities::RoleAbility, abilities_component::{ability::Ability, ability_id::AbilityID}, components::fast_forward::FastForwardComponent, event::{AsInvokable as _, Invokable as _}, role::Role
    }, lobby::{lobby_client::LobbyClient, Lobby}, log, packet::{ToClientPacket, ToServerPacket},
    room::{RemoveRoomClientResult, RoomClientID, RoomState}, vec_map::VecMap, websocket_connections::connection::ClientSender
};

use super::{
    event::on_game_ending::OnGameEnding,
    game_client::GameClientLocation,
    game_conclusion::GameConclusion,
    player::PlayerReference,
    role::RoleState,
    spectator::spectator_pointer::SpectatorPointer, Game
};



pub enum GameClientMessageResult {
    BackToLobby(Lobby),
    Close,
    None
}

impl Game {
    #[expect(clippy::match_single_binding, unused, reason="Surely spectators will do something in the future")]
    pub fn on_spectator_message(&mut self, sender_ref: SpectatorPointer, incoming_packet: ToServerPacket){
        match incoming_packet {
            _ => {}
        }
    }
    
    pub fn on_client_message(&mut self, _: &ClientSender, room_client_id: RoomClientID, incoming_packet: ToServerPacket) -> GameClientMessageResult {
        if let Some(client) = self.clients.get(&room_client_id) {
            match client.client_location {
                GameClientLocation::Player(player) => {
                    self.on_player_message(room_client_id, player, incoming_packet)
                }
                GameClientLocation::Spectator(spectator) => {
                    self.on_spectator_message(spectator, incoming_packet);
                    GameClientMessageResult::None
                }
            }
        } else {
            log!(error "Game"; "Received message from invalid client id: {}", room_client_id);
            GameClientMessageResult::None
        }
    }

    pub fn on_player_message(&mut self, room_client_id: RoomClientID, sender_player_ref: PlayerReference, incoming_packet: ToServerPacket) -> GameClientMessageResult {
        'packet_match: {match incoming_packet {
            ToServerPacket::Leave => {
                if let RemoveRoomClientResult::RoomShouldClose = self.remove_client(room_client_id) {
                    return GameClientMessageResult::Close;
                }
            },
            ToServerPacket::HostForceBackToLobby => {
                if let Some(player) = self.clients.get(&room_client_id) && !player.host {break 'packet_match}

                self.settings.role_list.simplify();
                let role_list = self.settings.role_list.clone();
                
                self.send_to_all(ToClientPacket::RoleList { role_list });

                let mut new_clients = VecMap::new();
                for (room_client_id, game_client) in self.clients.clone() {
                    new_clients.insert(room_client_id, LobbyClient::new_from_game_client(self, game_client));
                }

                self.send_to_all(ToClientPacket::BackToLobby);

                let lobby = Lobby::new_from_game(self.room_name.clone(), self.settings.clone(), new_clients);

                return GameClientMessageResult::BackToLobby(lobby);
            }
            ToServerPacket::HostForceEndGame => {
                if let Some(player) = self.clients.get(&room_client_id)
                    && !player.host {break 'packet_match}

                let conclusion = GameConclusion::get_premature_conclusion(self);

                OnGameEnding::new(conclusion).as_invokable().invoke(self);
            }
            ToServerPacket::HostForceSkipPhase => {
                if let Some(player) = self.clients.get(&room_client_id)
                    && !player.host {break 'packet_match}
                
                FastForwardComponent::skip(self);
            }
            ToServerPacket::HostDataRequest => {
                if let Some(player) = self.clients.get(&room_client_id) && !player.host {break 'packet_match}

                self.resend_host_data(sender_player_ref.connection(self));
            }
            ToServerPacket::HostForceSetPlayerName { id, name } => {
                if let Some(player) = self.clients.get(&room_client_id) && !player.host {break 'packet_match}
                if let Some(player) = self.clients.get(&id) && let GameClientLocation::Player(player) = player.client_location {
                    self.set_player_name(player, name);
                }
            }
            ToServerPacket::SetPlayerHost { player_id } => {
                if let Some(player) = self.clients.get(&room_client_id) && !player.host {break 'packet_match}
                if let Some(player) = self.clients.get_mut(&player_id) {
                    player.set_host();
                }
                self.send_players();
                self.resend_host_data_to_all_hosts();
            }
            ToServerPacket::RelinquishHost => {
                if let Some(player) = self.clients.get_mut(&room_client_id){
                    if !player.host {break 'packet_match}
                    player.relinquish_host();
                }
                self.ensure_host_exists(Some(room_client_id));
                self.send_players();
                self.resend_host_data_to_all_hosts();
            },
            ToServerPacket::SaveNotes { notes } => {
                sender_player_ref.set_notes(self, notes);
            },
            ToServerPacket::SaveCrossedOutOutlines { crossed_out_outlines } => {
                sender_player_ref.set_crossed_out_outlines(self, crossed_out_outlines);
            },
            ToServerPacket::SaveDeathNote { death_note } => {
                sender_player_ref.set_death_note(self, death_note);
            },
            ToServerPacket::ControllerInput { controller_input: ability_input } => 
                ability_input.on_client_message(self, sender_player_ref),
            ToServerPacket::SetConsortOptions { 
                roleblock, 
                you_were_roleblocked_message, 
                you_survived_attack_message, 
                you_were_guarded_message, 
                you_were_transported_message, 
                you_were_possessed_message, 
                you_were_wardblocked_message 
            } => {
                let id: AbilityID = AbilityID::Role { role: Role::Hypnotist, player: sender_player_ref };
                if let Some(Ability::Role(RoleAbility(RoleState::Hypnotist(mut hypnotist)))) = id.get_ability(self).cloned() {
                    hypnotist.roleblock = roleblock;

                    hypnotist.you_were_roleblocked_message = you_were_roleblocked_message;
                    hypnotist.you_survived_attack_message = you_survived_attack_message;
                    hypnotist.you_were_guarded_message = you_were_guarded_message;
                    hypnotist.you_were_transported_message = you_were_transported_message;
                    hypnotist.you_were_possessed_message = you_were_possessed_message;
                    hypnotist.you_were_wardblocked_message = you_were_wardblocked_message;

                    //There must be at least one message enabled, so if none are, enable roleblocked message
                    hypnotist.ensure_at_least_one_message();

                    id.edit_role_ability(self, hypnotist);
                }
            },
            ToServerPacket::VoteFastForwardPhase { fast_forward } => {
                sender_player_ref.set_fast_forward_vote(self, fast_forward);
            },
            ToServerPacket::WebRtcOffer { sdp: _ } => {
                // WebRTC offers should be handled by the room/lobby level
                log!(error "Game"; "WebRTC offer received in game state, should be handled at room level");
            },
            ToServerPacket::WebRtcIceCandidate { candidate: _, sdp_mid: _, sdp_m_line_index: _ } => {
                // WebRTC ICE candidates should be handled by the room/lobby level
                log!(error "Game"; "WebRTC ICE candidate received in game state, should be handled at room level");
            },
            _ => {
                log!(error "Game"; "Recieved invalid packet for Game state: {incoming_packet:?}");
            }
        }}
        
        for player_ref in PlayerReference::all_players(self){
            player_ref.send_repeating_data(self)
        }
        for spectator_ref in SpectatorPointer::all_spectators(self){
            spectator_ref.send_repeating_data(self)
        }

        GameClientMessageResult::None
    }
}