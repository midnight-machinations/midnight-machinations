use std::collections::HashMap;

use std::net::SocketAddr;

use super::{RoomCode, WebsocketListener, client::{Client, ClientLocation}};
use crate::{
    client_connection::ClientConnection,
    game::{
        abilities::role_abilities::RoleAbility,
        abilities_component::{Abilities, ability::Ability},
        Game,
        chat::{ChatGroup, PlayerChatGroupMap},
        components::silenced::Silenced,
        game_client::GameClientLocation,
        player::PlayerReference,
        role::RoleState,
    },
    lobby::{Lobby, lobby_client::LobbyClientType},
    packet::ToClientPacket,
    room::{Room, RoomClientID},
    voice::{VoicePeer, VoiceSession, random_token},
};

impl WebsocketListener {
    /// All event sources that can mutate rooms must use this boundary. Voice
    /// permissions are derived afresh after the entire action, including nested
    /// role events, controller inputs, and mid-phase status/group changes.
    /// Future timer, admin, or external event handlers must use this too.
    pub(super) fn with_voice_update<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R {
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
            let (mut peers, permissions) = match room.as_ref() {
                Room::Lobby(lobby) => lobby_voice(lobby, |id| bridge.in_voice(&scope, id)),
                Room::Game(game) => game_voice(game),
            };
            peers.sort_by_key(|peer| peer.id);
            let room_voice = RoomVoice { code: *code, url: &url, scope, peers, permissions };
            let mut members = room_voice.sessions(&mut self.clients, &mut policies);
            members.sort_by_key(|member| member.get("id").and_then(serde_json::Value::as_u64));
            rooms.push(serde_json::json!({"scope": room_voice.scope, "members": members}));
        }
        rooms.sort_by_key(|room| {
            room.get("scope")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        });
        if let Some(bridge) = &mut self.voice {
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

/// What one room client may do in voice
struct VoicePermission {
    send: Vec<String>,
    receive: Vec<String>,
    silenced: bool,
    connection: ClientConnection,
}

type VoicePermissions = HashMap<RoomClientID, VoicePermission>;

fn lobby_voice(lobby: &Lobby, in_voice: impl Fn(RoomClientID) -> bool) -> (Vec<VoicePeer>, VoicePermissions) {
    let mut peers = Vec::new();
    let mut permissions = HashMap::new();
    for (id, client) in lobby.clients.iter() {
        peers.push(VoicePeer {
            id: *id,
            name: match &client.client_type {
                LobbyClientType::Player { name } => name.clone(),
                LobbyClientType::Spectator { name } => name.clone(),
            },
            player_index: None,
            in_voice: in_voice(*id),
        });
        permissions.insert(*id, VoicePermission {
            send: vec!["lobby".to_owned()],
            receive: vec!["lobby".to_owned()],
            silenced: false,
            connection: client.connection.clone(),
        });
    }
    (peers, permissions)
}

fn game_voice(game: &Game) -> (Vec<VoicePeer>, VoicePermissions) {
    let receive = crate::game::chat::PlayerChatGroups::receive_player_chat_group_map(game);
    let mut peers = Vec::new();
    let mut permissions = HashMap::new();
    for (id, client) in game.clients.iter() {
        let (name, player_index, permission) = match client.client_location {
            GameClientLocation::Player(player) => (
                player.name(game).clone(),
                Some(player.index()),
                player_voice(game, player, &receive),
            ),
            GameClientLocation::Spectator(spectator) => (
                spectator.name(game),
                None,
                VoicePermission {
                    send: vec![],
                    receive: vec!["all".to_owned()],
                    silenced: false,
                    connection: game.spectators
                        .get(usize::from(spectator.index()))
                        .map(|s| s.connection.clone())
                        .unwrap_or(ClientConnection::Disconnected),
                },
            ),
        };
        peers.push(VoicePeer {
            id: *id,
            name,
            player_index,
            // Voice presence stays private during games.
            in_voice: false,
        });
        permissions.insert(*id, permission);
    }
    (peers, permissions)
}

fn chat_group_names<'a>(groups: impl IntoIterator<Item = &'a ChatGroup>) -> Vec<String> {
    groups
        .into_iter()
        .filter_map(|group| serde_json::to_value(group).ok()?.as_str().map(str::to_owned))
        .collect()
}

fn player_voice(game: &Game, player: PlayerReference, receive: &PlayerChatGroupMap) -> VoicePermission {
    let brained = Abilities::ids(game).iter().any(|id| matches!(
        id.get_ability(game),
        Some(Ability::Role(RoleAbility(RoleState::Cerenovous(state))))
            if state.currently_brained == Some(player)
    ));
    let silenced = brained || Silenced::silenced(game, player);
    VoicePermission {
        send: if silenced { vec![] } else { chat_group_names(&player.selected_send_chat_group(game)) },
        receive: chat_group_names(receive.get(player).iter()),
        silenced,
        connection: player.connection(game).clone(),
    }
}

/// One voice-enabled room's routing, used to build each member's session
struct RoomVoice<'a> {
    code: RoomCode,
    url: &'a str,
    scope: String,
    peers: Vec<VoicePeer>,
    permissions: VoicePermissions,
}

impl RoomVoice<'_> {
    /// Adds a session for each connected member to `policies` and returns the members for the control snapshot.
    fn sessions(&self, clients: &mut HashMap<SocketAddr, Client>, policies: &mut HashMap<SocketAddr, VoiceSession>) -> Vec<serde_json::Value> {
        let mut members = Vec::new();
        for (address, client) in clients {
            let ClientLocation::InRoom {
                room_code,
                room_client_id,
            } = client.location()
            else {
                continue;
            };
            if *room_code != self.code {
                continue;
            }
            let Some(VoicePermission { send, receive, silenced, connection: ClientConnection::Connected(connection) }) =
                self.permissions.get(room_client_id)
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
                .filter(|s| s.scope == self.scope && s.self_id == *room_client_id)
                .map(|s| s.token.clone())
                .unwrap_or_else(random_token);
            let session = VoiceSession {
                url: self.url.to_owned(),
                token: token.clone(),
                scope: self.scope.clone(),
                self_id: *room_client_id,
                send_groups: send.clone(),
                receive_groups: receive.clone(),
                forced_silenced: *silenced,
                peers: self.peers.clone(),
            };
            members.push(serde_json::json!({"id": room_client_id, "token": token, "send": send, "receive": receive}));
            policies.insert(*address, session);
        }
        members
    }
}
