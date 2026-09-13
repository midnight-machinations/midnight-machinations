//! Reading back the game logs written by [`crate::game::game_log`] so clients can watch a replay.
//!
//! Two things are exposed to clients: a cheap listing of every log in the logs directory
//! (used to draw the replay browser), and the full contents of a single log
//! (used to actually replay the game).

use std::{fs, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::log;

/// The directory game logs are written to and replayed from, relative to the server's working directory.
pub const LOGS_DIRECTORY: &str = "logs";

/// The maximum number of replays reported by [`list_replays`], newest first.
const MAX_LISTED_REPLAYS: usize = 500;

/// One row of the replay browser. Deliberately shaped like [`crate::packet::RoomPreviewData`]
/// so the client can draw it with the same table as the room browser.
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReplayPreviewData {
    /// Identifies the log, and is what the client sends back to request it. Never a path.
    pub file_name: String,
    pub room_name: String,
    pub started_at: String,
    pub ended_at: String,
    /// `None` for a log that ended without a recorded result.
    pub conclusion: Option<String>,
    /// Every player that was in the game, by player index.
    pub players: Vec<(u8, String)>,
}

/// Just enough of a recording to draw a row of the replay browser.
/// Everything else in the file (notably the timeline) is ignored.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecordingPreview {
    room_name: String,
    started_at: String,
    ended_at: String,
    setup: Option<RecordingPreviewSetup>,
    result: Option<RecordingPreviewResult>,
}

#[derive(Deserialize)]
struct RecordingPreviewSetup {
    players: Vec<RecordingPreviewPlayer>,
}

#[derive(Deserialize)]
struct RecordingPreviewPlayer {
    player: u8,
    name: String,
}

#[derive(Deserialize)]
struct RecordingPreviewResult {
    conclusion: String,
}

/// Every readable log in the logs directory, most recently started first.
///
/// Unreadable or malformed files are skipped rather than failing the whole listing, so one bad
/// file can't hide every replay.
pub fn list_replays() -> Vec<ReplayPreviewData> {
    let directory = PathBuf::from(LOGS_DIRECTORY);

    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        // A server that has never finished a game has no logs directory yet. That isn't an error.
        Err(error) => {
            if error.kind() != std::io::ErrorKind::NotFound {
                log!(error "Replay"; "Failed to read logs directory: {error}");
            }
            return Vec::new();
        }
    };

    let mut replays: Vec<ReplayPreviewData> = entries
        .filter_map(|entry| {
            let file_name = entry.ok()?.file_name().to_str()?.to_owned();
            if !is_valid_replay_file_name(&file_name) { return None; }
            read_preview(&directory.join(&file_name), file_name)
        })
        .collect();

    // Newest first, matching what someone looking for the game they just played expects.
    replays.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    replays.truncate(MAX_LISTED_REPLAYS);
    replays
}

fn read_preview(path: &Path, file_name: String) -> Option<ReplayPreviewData> {
    let contents = fs::read_to_string(path).ok()?;
    let preview: RecordingPreview = serde_json::from_str(&contents).ok()?;

    Some(ReplayPreviewData {
        file_name,
        room_name: preview.room_name,
        started_at: preview.started_at,
        ended_at: preview.ended_at,
        conclusion: preview.result.map(|result| result.conclusion),
        players: preview.setup
            .map(|setup| setup.players
                .into_iter()
                .map(|player| (player.player, player.name))
                .collect()
            )
            .unwrap_or_default(),
    })
}

/// The full contents of one log, returned verbatim so the client replays exactly what was written.
///
/// Returns `None` when `file_name` isn't a plain log file name, or the file can't be read or isn't
/// valid JSON.
pub fn read_replay(file_name: &str) -> Option<Box<RawValue>> {
    if !is_valid_replay_file_name(file_name) {
        log!(error "Replay"; "Rejected request for replay file name {file_name:?}");
        return None;
    }

    let contents = match fs::read_to_string(PathBuf::from(LOGS_DIRECTORY).join(file_name)) {
        Ok(contents) => contents,
        Err(error) => {
            log!(error "Replay"; "Failed to read replay {file_name}: {error}");
            return None;
        }
    };

    match RawValue::from_string(contents) {
        Ok(raw) => Some(raw),
        Err(error) => {
            log!(error "Replay"; "Replay {file_name} is not valid JSON: {error}");
            None
        }
    }
}

/// Whether `file_name` is a bare log file name and not a path.
///
/// Clients choose this name, so it has to be impossible to walk out of the logs directory with it:
/// only the characters [`crate::game::game_log`] can write are allowed, which excludes both
/// separators and `..`.
fn is_valid_replay_file_name(file_name: &str) -> bool {
    file_name.len() <= 255 &&
    !file_name.starts_with('.') &&
    file_name.ends_with(".json") &&
    file_name.len() > ".json".len() &&
    file_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') &&
    !file_name.contains("..")
}

#[cfg(test)]
mod tests {
    use super::is_valid_replay_file_name;

    #[test]
    fn accepts_names_the_game_log_writes() {
        assert!(is_valid_replay_file_name("Mafia_Room_20260913T071114.413Z.json"));
        assert!(is_valid_replay_file_name("room_20260101T000000.000Z.json"));
    }

    #[test]
    fn rejects_paths_and_traversal() {
        assert!(!is_valid_replay_file_name("../secrets.json"));
        assert!(!is_valid_replay_file_name(r"..\secrets.json"));
        assert!(!is_valid_replay_file_name("logs/a.json"));
        assert!(!is_valid_replay_file_name("/etc/passwd.json"));
        assert!(!is_valid_replay_file_name(r"C:\a.json"));
        assert!(!is_valid_replay_file_name("a..b.json"));
    }

    #[test]
    fn rejects_non_logs() {
        assert!(!is_valid_replay_file_name(""));
        assert!(!is_valid_replay_file_name(".json"));
        assert!(!is_valid_replay_file_name("game.txt"));
        assert!(!is_valid_replay_file_name("game"));
    }
}
