use std::{fs, path::PathBuf, time::Instant};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use crate::{log, replay::LOGS_DIRECTORY};

/// The full recording of a game, including the initial setup, the timeline of events and chat messages, and the final result.

use super::{
    chat::{ChatGroup, ChatMessageVariant},
    components::{insider_group::InsiderGroupID, synopsis::{Synopsis, SynopsisTracker}, win_condition::WinCondition},
    game_conclusion::GameConclusion,
    player::PlayerReference,
    role::ClientRoleStateEnum,
    settings::Settings,
    Game
};

/// Who a chat message was sent to.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ChatAudience {
    Group{group: ChatGroup},
    Player{player: PlayerReference},
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum GameLogContent {
    Event{event: &'static str, data: Value},
    ChatMessage{audience: ChatAudience, message: ChatMessageVariant},
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameLogEntry {
    pub timestamp: DateTime<Utc>,
    pub elapsed_ms: u64,
    #[serde(flatten)]
    pub content: GameLogContent,
}

/// Every event and chat message that has occurred in a game so far (+timestamp).
pub struct GameLog {
    started_at: DateTime<Utc>,
    start_instant: Instant,
    setup: Option<GameSetup>,
    entries: Vec<GameLogEntry>,
}
impl Default for GameLog {
    fn default() -> Self {
        Self {
            started_at: Utc::now(),
            start_instant: Instant::now(),
            setup: None,
            entries: Vec::new(),
        }
    }
}
impl GameLog {
    pub fn new() -> Self {
        Self::default()
    }

    fn elapsed_ms(&self) -> u64 {
        u64::try_from(self.start_instant.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    /// Whether `set_setup` has been called yet. This is because I wanted to silence events that are
    /// noisy during initial setup like when abilities are assigned with the current system.
    pub fn is_initialized(&self) -> bool {
        self.setup.is_some()
    }

    /// the game's initial setup (settings, players, assigned roles).
    pub fn set_setup(&mut self, setup: GameSetup) {
        self.setup = Some(setup);
    }

    pub fn push_event(&mut self, event: &'static str, data: Value) {
        self.entries.push(GameLogEntry {
            timestamp: Utc::now(),
            elapsed_ms: self.elapsed_ms(),
            content: GameLogContent::Event { event, data },
        });
    }

    pub fn push_chat_message(&mut self, audience: ChatAudience, message: ChatMessageVariant) {
        self.entries.push(GameLogEntry {
            timestamp: Utc::now(),
            elapsed_ms: self.elapsed_ms(),
            content: GameLogContent::ChatMessage { audience, message },
        });
    }

    /// Serializes the full recording (setup, timeline, result) and
    /// writes it to `logs/<room_name>_<started_at>.json`.
    /// The file written here is exactly what the replay feature reads back, see [`crate::replay`].
    pub fn write_to_disk(&self, room_name: &str, result: GameResult) {
        let recording = GameRecording {
            room_name,
            started_at: self.started_at,
            ended_at: Utc::now(),
            setup: self.setup.as_ref(),
            timeline: &self.entries,
            result,
        };

        let json = match serde_json::to_string_pretty(&recording) {
            Ok(json) => json,
            Err(error) => {
                log!(error "GameLog"; "Failed to serialize game log for room {room_name}: {error}");
                return;
            }
        };

        let dir = PathBuf::from(LOGS_DIRECTORY);
        if let Err(error) = fs::create_dir_all(&dir) {
            log!(error "GameLog"; "Failed to create logs directory: {error}");
            return;
        }

        let file_name = format!(
            "{}_{}.json",
            sanitize_file_name(room_name),
            self.started_at.format("%Y%m%dT%H%M%S%.3fZ")
        );

        if let Err(error) = fs::write(dir.join(file_name), json) {
            log!(error "GameLog"; "Failed to write game log for room {room_name}: {error}");
        }
    }
}

fn sanitize_file_name(name: &str) -> String {
    let sanitized: String = name.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    if sanitized.is_empty() { "room".to_string() } else { sanitized }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GameRecording<'a> {
    room_name: &'a str,
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
    setup: Option<&'a GameSetup>,
    timeline: &'a [GameLogEntry],
    result: GameResult,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSnapshot {
    pub player: PlayerReference,
    pub name: String,
    pub alive: bool,
    pub role: ClientRoleStateEnum,
    pub insider_groups: Vec<InsiderGroupID>,
    pub win_condition: WinCondition,
}
impl PlayerSnapshot {
    fn capture(game: &Game, player: PlayerReference) -> Self {
        Self {
            player,
            name: player.name(game).clone(),
            alive: player.alive(game),
            role: player.role_state(game).clone().get_client_ability_state(game, player),
            insider_groups: InsiderGroupID::all_groups_with_player(game, player).iter().copied().collect(),
            win_condition: player.win_condition(game).clone(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSetup {
    pub settings: Settings,
    pub players: Vec<PlayerSnapshot>,
}
impl GameSetup {
    pub fn capture(game: &Game) -> Self {
        Self {
            settings: game.settings.clone(),
            players: PlayerReference::all_players(game).map(|player| PlayerSnapshot::capture(game, player)).collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameResult {
    pub conclusion: GameConclusion,
    pub synopsis: Synopsis,
    pub players: Vec<PlayerSnapshot>,
}
impl GameResult {
    pub fn capture(game: &Game, conclusion: GameConclusion) -> Self {
        Self {
            conclusion,
            synopsis: SynopsisTracker::get(game, conclusion),
            players: PlayerReference::all_players(game).map(|player| PlayerSnapshot::capture(game, player)).collect(),
        }
    }
}
