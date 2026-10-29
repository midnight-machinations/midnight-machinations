use std::collections::HashMap;

use super::{WebsocketListener, client::ClientLocation};
use crate::{
    client_connection::ClientConnection,
    game::{
        abilities::role_abilities::RoleAbility,
        abilities_component::{Abilities, ability::Ability},
        chat::PlayerChatGroups,
        components::silenced::Silenced,
        game_client::GameClientLocation,
        role::RoleState,
    },
    lobby::lobby_client::LobbyClientType,
    packet::ToClientPacket,
    room::Room,
    voice::{VoicePeer, VoiceSession, random_token},
};

impl WebsocketListener {
    /// All event sources that can mutate rooms must use this boundary. Voice
    /// permissions are derived afresh after the entire action, including nested
    /// role events, controller inputs, and mid-phase status/group changes.
    /// Future timer, admin, or external event handlers must use this too.
    pub(super) fn with_voice_update<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R {
        if let Some(voice) = &self.voice {
            voice.freeze();
        }
        let result = action(self);
        self.sync_voice();
        result
    }

    pub(super) fn sync_voice(&mut self) {
        let Some(bridge) = &self.voice else { return };
        let url = bridge.public_url.clone();
        self.voice_scopes
            .retain(|code, _| self.rooms.contains_key(code));
        let mut policies = HashMap::new();
        let mut rooms = Vec::new();
        for (code, room) in &self.rooms {
            let enabled = match room.as_ref() {
                Room::Lobby(lobby) => lobby.settings.voice_enabled,
                Room::Game(game) => game.settings.voice_enabled,
            };
            // Disabled rooms publish no media credentials or forwarding routes.
            // Existing media sessions are revoked by the next control snapshot.
            if !enabled { continue; }
            let in_game = matches!(room.as_ref(), Room::Game(_));
            let scope = self
                .voice_scopes
                .entry(*code)
                .or_insert_with(|| (in_game, random_token()));
            if scope.0 != in_game {
                *scope = (in_game, random_token());
            }
            let scope = scope.1.clone();
            let mut peers = Vec::new();
            let mut permissions = HashMap::new();
            match room.as_ref() {
                Room::Lobby(lobby) => {
                    for (id, client) in lobby.clients.iter() {
                        peers.push(VoicePeer {
                            id: *id,
                            name: match &client.client_type {
                                LobbyClientType::Player { name } => name.clone(),
                                LobbyClientType::Spectator => format!("Spectator {id}"),
                            },
                            player_index: None,
                            in_voice: bridge.in_voice(&scope, *id),
                        });
                        permissions.insert(
                            *id,
                            (
                                vec!["lobby".to_owned()],
                                vec!["lobby".to_owned()],
                                false,
                                client.connection.clone(),
                            ),
                        );
                    }
                }
                Room::Game(game) => {
                    let send = PlayerChatGroups::compute_send_map(game);
                    let receive = PlayerChatGroups::receive_player_chat_group_map(game);
                    for (id, client) in game.clients.iter() {
                        let (name, index, sends, receives, silenced, connection) =
                            match client.client_location {
                                GameClientLocation::Player(player) => {
                                    let brained = Abilities::ids(game).iter().any(|id| matches!(
                                    id.get_ability(game),
                                    Some(Ability::Role(RoleAbility(RoleState::Cerenovous(state))))
                                        if state.currently_brained == Some(player)
                                ));
                                    let silenced = brained || Silenced::silenced(game, player);
                                    let groups = |map: &crate::game::chat::PlayerChatGroupMap| {
                                        map.get(player)
                                            .iter()
                                            .filter_map(|group| {
                                                serde_json::to_value(group)
                                                    .ok()?
                                                    .as_str()
                                                    .map(str::to_owned)
                                            })
                                            .collect::<Vec<_>>()
                                    };
                                    (
                                        player.name(game).clone(),
                                        Some(player.index()),
                                        if silenced { vec![] } else { groups(&send) },
                                        groups(&receive),
                                        silenced,
                                        player.connection(game).clone(),
                                    )
                                }
                                GameClientLocation::Spectator(spectator) => (
                                    format!("Spectator {id}"),
                                    None,
                                    vec![],
                                    vec!["all".to_owned()],
                                    false,
                                    game.spectators
                                        .get(usize::from(spectator.index()))
                                        .map(|s| s.connection.clone())
                                        .unwrap_or(ClientConnection::Disconnected),
                                ),
                            };
                        peers.push(VoicePeer {
                            id: *id,
                            name,
                            player_index: index,
                            // Voice presence stays private during games.
                            in_voice: false,
                        });
                        permissions.insert(*id, (sends, receives, silenced, connection));
                    }
                }
            }
            peers.sort_by_key(|peer| peer.id);
            let mut members = Vec::new();
            for (address, client) in &mut self.clients {
                let ClientLocation::InRoom {
                    room_code,
                    room_client_id,
                } = client.location()
                else {
                    continue;
                };
                if room_code != code {
                    continue;
                }
                let Some((send, receive, silenced, ClientConnection::Connected(connection))) =
                    permissions.get(room_client_id)
                else {
                    continue;
                };
                // A replaced game socket cannot retain an independent voice session.
                if !connection.same_connection(&client.sender()) {
                    continue;
                }
                let token = client
                    .voice_session
                    .as_ref()
                    .filter(|s| s.scope == scope && s.self_id == *room_client_id)
                    .map(|s| s.token.clone())
                    .unwrap_or_else(random_token);
                let session = VoiceSession {
                    url: url.clone(),
                    token: token.clone(),
                    scope: scope.clone(),
                    self_id: *room_client_id,
                    send_groups: send.clone(),
                    receive_groups: receive.clone(),
                    forced_silenced: *silenced,
                    peers: peers.clone(),
                };
                members.push(serde_json::json!({"id": room_client_id, "token": token, "send": send, "receive": receive}));
                policies.insert(*address, session);
            }
            members.sort_by_key(|member| member.get("id").and_then(serde_json::Value::as_u64));
            rooms.push(serde_json::json!({"scope": scope, "members": members}));
        }
        rooms.sort_by_key(|room| {
            room.get("scope")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        });
        if let Some(bridge) = &self.voice {
            bridge.publish(serde_json::json!(rooms));
        }
        for (address, client) in &mut self.clients {
            let next = policies.remove(address);
            if client.voice_session != next {
                client.send(ToClientPacket::VoiceSession {
                    session: next.clone(),
                });
                client.voice_session = next;
            }
        }
    }
}
