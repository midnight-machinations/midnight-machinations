mod kit;

use mafia_server::game::{
    game_conclusion::GameConclusion,
    game_log::{GameResult, GameSetup},
    prelude::*,
};

/// The game log is automatically filled as the game runs normally, so this test just runs a game then checks the log.
#[test]
fn game_log_records_setup_events_chat_and_result_as_valid_json() {
    kit::scenario!(game in Night 1 where
        sher: Detective,
        mafia: Mafioso,
        _townie: Detective
    );

    sher.send_ability_input_player_list_typical(mafia);
    game.next_phase();

    assert!(!sher.get_messages_after_night(1).is_empty());

    let setup = GameSetup::capture(&game);
    game.game_log.set_setup(setup);

    let result = GameResult::capture(&game, GameConclusion::Town);
    game.game_log.write_to_disk("game_log_test_room", result);

    let logs_dir = std::path::Path::new("logs");
    let mut matching_files = std::fs::read_dir(logs_dir)
        .expect("logs directory should have been created")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("game_log_test_room_"))
        })
        .collect::<Vec<_>>();

    let path = matching_files.pop().expect("write_to_disk should have written a file for this room");

    let contents = std::fs::read_to_string(&path).expect("written game log should be readable");
    let json: serde_json::Value = serde_json::from_str(&contents).expect("written game log should be valid JSON");

    assert_eq!(json["roomName"], "game_log_test_room");
    assert!(json["setup"]["players"].as_array().is_some_and(|players| players.len() == 3));
    assert_eq!(json["result"]["conclusion"], "town");

    let timeline = json["timeline"].as_array().expect("timeline should be an array");
    assert!(!timeline.is_empty());
    assert!(timeline.iter().any(|entry| entry["kind"] == "chatMessage"));
    assert!(timeline.iter().any(|entry|
        entry["kind"] == "event" && entry["event"] == "OnControllerInputReceived"
    ));

    assert!(timeline.iter().all(|entry| entry["event"] != "OnTick"));

    std::fs::remove_file(&path).expect("should be able to clean up the test's own log file");
}
