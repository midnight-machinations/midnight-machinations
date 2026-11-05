#[allow(dead_code)]
mod kit;

use mafia_server::{
    game::{on_client_message::GameClientMessageResult, phase::PhaseType, role::Role},
    packet::{ToClientPacket, ToServerPacket},
    room::RoomState,
    websocket_connections::connection::{ClientSender, Connection},
};

fn connection() -> ClientSender {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel::<ToClientPacket>();
    Connection::new(tx, "127.0.0.1:0".parse().unwrap()).sender()
}

fn scenario() -> kit::TestScenario {
    kit::_init::create_basic_scenario(vec![Role::Detective, Role::Detective, Role::Mafioso])
}

#[test]
fn spectator_host_can_end_the_game() {
    let mut scenario = scenario();
    let game = &mut scenario.game;
    let sender = connection();
    let host = game.join_client(&sender).unwrap();
    assert!(host.spectator && game.is_host(host.id));

    let _ = game.on_client_message(&sender, host.id, ToServerPacket::HostForceEndGame);
    assert_eq!(game.current_phase().phase(), PhaseType::Recess);
}

#[test]
fn spectator_host_can_return_to_lobby() {
    let mut scenario = scenario();
    let game = &mut scenario.game;
    let sender = connection();
    let host = game.join_client(&sender).unwrap();

    let result = game.on_client_message(&sender, host.id, ToServerPacket::HostForceBackToLobby);
    assert!(matches!(result, GameClientMessageResult::BackToLobby(..)));
}

#[test]
fn spectator_host_can_pass_and_relinquish_host() {
    let mut scenario = scenario();
    let game = &mut scenario.game;
    let sender = connection();
    let host = game.join_client(&sender).unwrap();
    let guest = game.join_client(&sender).unwrap();
    assert!(!game.is_host(guest.id));

    let _ = game.on_client_message(&sender, host.id, ToServerPacket::SetPlayerHost { player_id: guest.id });
    assert!(game.is_host(guest.id));

    let _ = game.on_client_message(&sender, host.id, ToServerPacket::RelinquishHost);
    assert!(!game.is_host(host.id));
    assert!(game.is_host(guest.id));
}

#[test]
fn non_host_spectator_cannot_use_host_commands() {
    let mut scenario = scenario();
    let game = &mut scenario.game;
    let sender = connection();
    let _host = game.join_client(&sender).unwrap();
    let guest = game.join_client(&sender).unwrap();

    let result = game.on_client_message(&sender, guest.id, ToServerPacket::HostForceBackToLobby);
    assert!(matches!(result, GameClientMessageResult::None));
    let _ = game.on_client_message(&sender, guest.id, ToServerPacket::HostForceEndGame);
    assert_eq!(game.current_phase().phase(), PhaseType::Briefing);
    let _ = game.on_client_message(&sender, guest.id, ToServerPacket::SetPlayerHost { player_id: guest.id });
    assert!(!game.is_host(guest.id));
}
