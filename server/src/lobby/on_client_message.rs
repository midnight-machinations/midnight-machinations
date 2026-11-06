use std::collections::VecDeque;

use crate::{
    game::{
        chat::{ChatMessage, ChatMessageVariant},
        game_client::{GameClient, GameClientLocation},
        phase::PhaseType, player::{PlayerIndex, PlayerInitializeParameters, PlayerReference},
        spectator::{spectator_pointer::{SpectatorIndex, SpectatorPointer},
        SpectatorInitializeParameters}, Game, RejectStartReason
    }, 
    log, packet::{ToClientPacket, ToServerPacket}, 
    room::{name_validation::{self, sanitize_server_name}, RemoveRoomClientResult,
    RoomClientID, RoomState}, strings::TidyableString, vec_map::{vec_map, VecMap},
    websocket_connections::connection::ClientSender
};

use super::{lobby_client::{LobbyClient, LobbyClientType, Ready}, Lobby};

pub enum LobbyClientMessageResult {
    StartGame(Box<Game>),
    Close,
    None
}

impl Lobby {
    pub fn on_client_message(&mut self, send: &ClientSender, room_client_id: RoomClientID, incoming_packet: ToServerPacket) -> LobbyClientMessageResult {
        let host_only = packet_is_host_only(&incoming_packet);
        if host_only && self.sent_by_non_host(room_client_id) {
            return LobbyClientMessageResult::None;
        }

        match incoming_packet {
            ToServerPacket::SendLobbyMessage { text } => self.send_lobby_message(room_client_id, text),
            ToServerPacket::SetSpectator { spectator } => self.set_spectator(room_client_id, spectator),
            ToServerPacket::SetName{ name } => self.set_player_name(room_client_id, name),
            ToServerPacket::ReadyUp{ ready } => self.ready_up(room_client_id, ready),
            ToServerPacket::SetRoomName{ name } => self.set_room_name(name),
            ToServerPacket::StartGame => {
                if let Some(game) = self.start_game(send) {
                    return LobbyClientMessageResult::StartGame(Box::new(game));
                }
            },
            ToServerPacket::SetPhaseTime{phase, time} => self.set_phase_time(phase, time),
            ToServerPacket::SetPhaseTimes { phase_time_settings } => {
                self.settings.phase_times = phase_time_settings.clone();
                self.send_to_all(ToClientPacket::PhaseTimes { phase_time_settings });
            }
            ToServerPacket::SetRoleList { role_list } => {
                self.settings.role_list = role_list;
                self.set_rolelist_length();
                let role_list = self.settings.role_list.clone();
                self.send_to_all(ToClientPacket::RoleList { role_list });
            }
            ToServerPacket::SetRandomSeed { random_seed } => {
                self.settings.random_seed = random_seed;
                self.send_to_all(ToClientPacket::RandomSeed { random_seed });
            }
            ToServerPacket::SetRoleOutline { index, role_outline } => {
                let Some(unset_outline) = self.settings.role_list.0.get_mut(index as usize) else { return LobbyClientMessageResult::None };
                *unset_outline = role_outline.clone();
                self.send_to_all(ToClientPacket::RoleOutline { index, role_outline });
            }
            ToServerPacket::SimplifyRoleList => {
                self.settings.role_list.simplify();
                let role_list = self.settings.role_list.clone();
                self.send_to_all(ToClientPacket::RoleList { role_list });
            }
            ToServerPacket::SetEnabledRoles { roles } => {
                self.settings.enabled_roles = roles.into_iter().collect();
                let roles = self.settings.enabled_roles.clone().into_iter().collect();
                self.send_to_all(ToClientPacket::EnabledRoles { roles });
            }
            ToServerPacket::SetModifierSettings { modifier_settings } => {
                self.settings.modifiers = modifier_settings.clone();
                self.send_to_all(ToClientPacket::ModifierSettings { modifier_settings });
            }
            ToServerPacket::Leave => {
                if let RemoveRoomClientResult::RoomShouldClose = self.remove_client(room_client_id) {
                    return LobbyClientMessageResult::Close;
                }
            }
            ToServerPacket::HostForceSetPlayerName { id, name } => self.set_player_name(id, name),
            ToServerPacket::SetPlayerHost { player_id } => {
                if let Some(player) = self.clients.get_mut(&player_id) {
                    player.set_host();
                }
                self.send_players();
            }
            ToServerPacket::RelinquishHost => {
                if let Some(player) = self.clients.get_mut(&room_client_id) {
                    if !player.is_host() { return LobbyClientMessageResult::None }
                    player.relinquish_host();
                }
                self.ensure_host_exists(Some(room_client_id));
                self.send_players();
            }
            _ => {
                log!(error "Lobby"; "{} {:?}", "ToServerPacket not implemented for lobby was sent during lobby: ", incoming_packet);
            }
        }

        LobbyClientMessageResult::None
    }

    /// True when the sender is a known client who is not a host. Unknown senders are let through, as before.
    fn sent_by_non_host(&self, room_client_id: RoomClientID) -> bool {
        matches!(self.clients.get(&room_client_id), Some(client) if !client.is_host())
    }

    fn send_lobby_message(&mut self, room_client_id: RoomClientID, text: String) {
        let text = text.trim_newline().trim_whitespace().truncate(100);
        if text.is_empty() { return }

        let Some(LobbyClient { client_type: LobbyClientType::Player { name }, .. }) = self.clients.get(&room_client_id) else { return };
        let name = name.clone();

        self.send_to_all(ToClientPacket::AddChatMessages { chat_messages: vec_map![(
            self.chat_message_index,
            ChatMessage::new_non_private(
                ChatMessageVariant::LobbyMessage { sender: name, text },
                crate::game::chat::ChatGroup::All
            )
        )]});
        self.chat_message_index = self.chat_message_index.saturating_add(1);
    }

    fn set_spectator(&mut self, room_client_id: RoomClientID, spectator: bool) {
        if let Some(client) = self.clients.get_mut(&room_client_id) {
            let name = client.name().clone();
            client.client_type = if spectator {
                LobbyClientType::Spectator { name }
            } else {
                LobbyClientType::Player { name }
            };
        }

        self.set_rolelist_length();
        self.send_players();
        let role_list = self.settings.role_list.clone();
        self.send_to_all(ToClientPacket::RoleList { role_list } );
    }

    fn ready_up(&mut self, room_client_id: RoomClientID, ready: bool) {
        if let Some(player) = self.clients.get_mut(&room_client_id) && player.ready != Ready::Host {
            player.ready = if ready { Ready::Ready } else { Ready::NotReady }
        }

        let ready = self.clients.iter()
            .filter(|(_, client)| client.ready == Ready::Ready)
            .map(|(id, _)| *id)
            .collect();
        self.send_to_all(ToClientPacket::PlayersReady { ready });
    }

    fn set_room_name(&mut self, name: String) {
        let name = sanitize_server_name(name);
        let name = if name.is_empty() {
            name_validation::DEFAULT_SERVER_NAME.to_string()
        } else {
            name
        };

        self.name.clone_from(&name);

        self.send_to_all(ToClientPacket::RoomName { name })
    }

    fn set_phase_time(&mut self, phase: PhaseType, time: u16) {
        match phase {
            PhaseType::Briefing => { self.settings.phase_times.briefing = time; }
            PhaseType::Obituary => { self.settings.phase_times.obituary = time; }
            PhaseType::Discussion => { self.settings.phase_times.discussion = time; }
            PhaseType::FinalWords => { self.settings.phase_times.final_words = time; }
            PhaseType::Dusk => { self.settings.phase_times.dusk = time; }
            PhaseType::Judgement => { self.settings.phase_times.judgement = time; }
            PhaseType::Night => { self.settings.phase_times.night = time; }
            PhaseType::Testimony => { self.settings.phase_times.testimony = time; }
            PhaseType::Nomination => { self.settings.phase_times.nomination = time; }
            PhaseType::Adjournment | PhaseType::Recess => { }
        };

        self.send_to_all(ToClientPacket::PhaseTime { phase, time });
    }

    /// Builds the game from the lobby, or tells the host why it cannot start
    fn start_game(&mut self, send: &ClientSender) -> Option<Game> {
        let mut game_clients: VecMap<RoomClientID, GameClient> = VecMap::new();
        let mut game_player_params = Vec::new();
        let mut game_spectator_params = Vec::new();

        let mut next_player_index: PlayerIndex = 0;
        let mut next_spectator_index: SpectatorIndex = 0;

        for (room_client_id, lobby_client) in self.clients.clone() {
            let client_location = if let LobbyClientType::Spectator { .. } = lobby_client.client_type {
                GameClientLocation::Spectator(SpectatorPointer::new(next_spectator_index))
            } else {
                GameClientLocation::Player(unsafe { PlayerReference::new_unchecked(next_player_index) })
            };
            game_clients.insert(room_client_id, GameClient {
                client_location,
                host: lobby_client.is_host(),
                last_message_times: VecDeque::new(),
            });

            let next_index = match lobby_client.client_type {
                LobbyClientType::Player { ref name } => {
                    game_player_params.push(PlayerInitializeParameters{
                        host: lobby_client.is_host(),
                        connection: lobby_client.connection,
                        name: name.clone(),
                    });
                    next_player_index.checked_add(1).map(|index| next_player_index = index)
                },
                LobbyClientType::Spectator { ref name } => {
                    game_spectator_params.push(SpectatorInitializeParameters{
                        name: name.clone(),
                        host: lobby_client.is_host(),
                        connection: lobby_client.connection,
                    });
                    next_spectator_index.checked_add(1).map(|index| next_spectator_index = index)
                }
            };
            if next_index.is_none() {
                send.send(ToClientPacket::RejectStart { reason: RejectStartReason::TooManyClients });
                return None;
            }
        }

        let game = match Game::new(self.name.clone(), self.settings.clone(), game_clients, game_player_params, game_spectator_params){
            Ok(game) => game,
            Err(err) => {
                send.send(ToClientPacket::RejectStart { reason: err });
                log!(info "Lobby"; "Failed to start game: {:?}", err);
                return None
            }
        };

        self.send_to_all(ToClientPacket::RoomName { name: self.name.clone() });

        Some(game)
    }
}

/// Packets that only a host may send while in the lobby
fn packet_is_host_only(packet: &ToServerPacket) -> bool {
    matches!(packet,
        ToServerPacket::SetRoomName { .. }
        | ToServerPacket::StartGame
        | ToServerPacket::SetPhaseTime { .. }
        | ToServerPacket::SetPhaseTimes { .. }
        | ToServerPacket::SetRoleList { .. }
        | ToServerPacket::SetRandomSeed { .. }
        | ToServerPacket::SetRoleOutline { .. }
        | ToServerPacket::SimplifyRoleList
        | ToServerPacket::SetEnabledRoles { .. }
        | ToServerPacket::SetModifierSettings { .. }
        | ToServerPacket::HostForceSetPlayerName { .. }
        | ToServerPacket::SetPlayerHost { .. }
    )
}
