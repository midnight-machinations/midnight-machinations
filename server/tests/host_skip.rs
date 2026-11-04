#[allow(dead_code)]
mod kit;

use std::time::Duration;

use mafia_server::{
    game::{
        components::fast_forward::{FastForwardComponent, FastForwardSetting},
        phase::PhaseType,
        role::Role,
    },
    packet::{ToClientPacket, ToServerPacket},
    room::RoomState,
    websocket_connections::connection::Connection,
};

#[test]
fn spectator_host_skips_to_selected_phase_and_day() {
    let mut scenario = kit::_init::create_basic_scenario(vec![Role::Detective, Role::Detective, Role::Mafioso]);
    let game = &mut scenario.game;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let sender = Connection::new(tx, "127.0.0.1:0".parse().unwrap()).sender();
    let host = game.join_client(&sender).unwrap();
    assert!(host.spectator);
    assert!(game.is_host(host.id));

    let setting = FastForwardSetting::Phase { phase: PhaseType::Discussion, day: 3 };
    let packet = serde_json::from_str::<ToServerPacket>(
        r#"{"type":"hostForceSkipPhase","fastForward":{"type":"phase","phase":"discussion","day":3}}"#
    ).unwrap();
    let _ = game.on_client_message(&sender, host.id, packet);
    assert_eq!(game.host_fast_forward, setting);
    for player in &scenario.players {
        assert_eq!(player.fast_forward_vote(game), FastForwardSetting::None);
    }
    let _ = game.tick(Duration::ZERO);
    assert_eq!(game.current_phase().phase(), PhaseType::Discussion);
    assert_eq!(game.day_number(), 3);
    assert_eq!(game.host_fast_forward, FastForwardSetting::None);
    let _ = game.tick(Duration::ZERO);
    assert_eq!(game.current_phase().phase(), PhaseType::Discussion);

    let mut host_settings = Vec::new();
    while let Ok(packet) = rx.try_recv() {
        if let ToClientPacket::HostData { fast_forward, .. } = packet {
            host_settings.push(fast_forward);
        }
    }
    assert!(host_settings.contains(&setting));
    assert_eq!(host_settings.last(), Some(&FastForwardSetting::None));
}

#[test]
fn non_host_cannot_force_skip() {
    let mut scenario = kit::_init::create_basic_scenario(vec![Role::Detective, Role::Detective, Role::Mafioso]);
    let game = &mut scenario.game;
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let sender = Connection::new(tx, "127.0.0.1:0".parse().unwrap()).sender();
    let _host = game.join_client(&sender).unwrap();
    let guest = game.join_client(&sender).unwrap();
    assert!(!game.is_host(guest.id));
    let _ = game.on_client_message(&sender, guest.id, ToServerPacket::HostForceSkipPhase {
        fast_forward: FastForwardSetting::Phase { phase: PhaseType::Night, day: 2 },
    });
    let _ = game.tick(Duration::ZERO);
    assert_eq!(game.current_phase().phase(), PhaseType::Briefing);
    assert_eq!(game.host_fast_forward, FastForwardSetting::None);
}

#[test]
fn host_can_skip_just_one_phase() {
    let mut scenario = kit::_init::create_basic_scenario(vec![Role::Detective, Role::Detective, Role::Mafioso]);
    let game = &mut scenario.game;
    FastForwardComponent::host_skip(game, FastForwardSetting::Skip);
    let _ = game.tick(Duration::ZERO);
    assert_eq!(game.current_phase().phase(), PhaseType::Dusk);
    assert_eq!(game.host_fast_forward, FastForwardSetting::None);
    let _ = game.tick(Duration::ZERO);
    assert_eq!(game.current_phase().phase(), PhaseType::Dusk);
}

#[test]
fn host_can_cancel_target() {
    let mut scenario = kit::_init::create_basic_scenario(vec![Role::Detective, Role::Detective, Role::Mafioso]);
    let game = &mut scenario.game;
    FastForwardComponent::host_skip(game, FastForwardSetting::Phase { phase: PhaseType::Night, day: 2 });
    FastForwardComponent::host_skip(game, FastForwardSetting::None);
    assert_eq!(game.host_fast_forward, FastForwardSetting::None);
    let _ = game.tick(Duration::ZERO);
    assert_eq!(game.current_phase().phase(), PhaseType::Dusk);
}

#[test]
fn host_skip_rejects_unreachable_and_past_targets() {
    let mut scenario = kit::_init::create_basic_scenario(vec![Role::Detective, Role::Detective, Role::Mafioso]);
    let game = &mut scenario.game;
    for (phase, day) in [(PhaseType::Judgement, 2), (PhaseType::Discussion, 1), (PhaseType::Night, 255)] {
        FastForwardComponent::host_skip(game, FastForwardSetting::Phase { phase, day });
        let _ = game.tick(Duration::ZERO);
        assert_eq!(game.current_phase().phase(), PhaseType::Briefing);
        assert_eq!(game.host_fast_forward, FastForwardSetting::None);
    }
    FastForwardComponent::host_skip(game, FastForwardSetting::Phase { phase: PhaseType::Night, day: 1 });
    let _ = game.tick(Duration::ZERO);
    FastForwardComponent::host_skip(game, FastForwardSetting::Phase { phase: PhaseType::Night, day: 1 });
    let _ = game.tick(Duration::ZERO);
    assert_eq!(game.current_phase().phase(), PhaseType::Night);
    assert_eq!(game.day_number(), 1);
}
