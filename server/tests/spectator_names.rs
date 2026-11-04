use mafia_server::{
    game::{role::Role, role_list::{RoleList, RoleOutline}, spectator::spectator_pointer::SpectatorPointer},
    lobby::{Lobby, lobby_client::LobbyClientType, on_client_message::LobbyClientMessageResult},
    packet::{ToClientPacket, ToServerPacket},
    room::{RoomState, name_validation::sanitize_name},
    websocket_connections::connection::{ClientSender, Connection},
};

fn connection() -> (ClientSender, tokio::sync::mpsc::UnboundedReceiver<ToClientPacket>) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    (Connection::new(tx, "127.0.0.1:0".parse().unwrap()).sender(), rx)
}

#[test]
fn switching_preserves_the_current_name_and_spectators_can_rename_themselves() {
    let (sender, _rx) = connection();
    let mut lobby = Lobby::new();
    let id = lobby.join_client(&sender).unwrap().id;
    lobby.set_player_name(id, "Alice".into());
    lobby.on_client_message(&sender, id, ToServerPacket::SetSpectator { spectator: true });
    assert!(matches!(&lobby.clients.get(&id).unwrap().client_type,
        LobbyClientType::Spectator { name } if name == "Alice"));

    lobby.on_client_message(&sender, id, ToServerPacket::SetName { name: "Observer".into() });
    lobby.on_client_message(&sender, id, ToServerPacket::SetSpectator { spectator: false });
    assert!(matches!(&lobby.clients.get(&id).unwrap().client_type,
        LobbyClientType::Player { name } if name == "Observer"));
    lobby.on_client_message(&sender, id, ToServerPacket::SetSpectator { spectator: true });
    assert_eq!(lobby.clients.get(&id).unwrap().name(), "Observer");
}

#[test]
fn spectator_submissions_use_player_validation_and_reserve_their_names() {
    let (sender, _rx) = connection();
    let mut lobby = Lobby::new();
    let host = lobby.join_client(&sender).unwrap().id;
    let guest = lobby.join_client(&sender).unwrap().id;
    lobby.on_client_message(&sender, guest, ToServerPacket::SetSpectator { spectator: true });
    let desired = "  ABCDEFGHIJKLMNOPQRSTUVWXYZ\n\r\t  ".to_string();
    let expected = sanitize_name(desired.clone(), &[]);
    lobby.on_client_message(&sender, guest, ToServerPacket::SetName { name: desired });
    assert_eq!(lobby.clients.get(&guest).unwrap().name(), &expected);
    assert!(expected.len() <= 20);
    assert!(!expected.contains(['\n', '\r']));

    lobby.set_player_name(host, expected.clone());
    assert_ne!(lobby.clients.get(&host).unwrap().name(), &expected);
    lobby.on_client_message(&sender, host, ToServerPacket::HostForceSetPlayerName { id: guest, name: "Watcher".into() });
    assert_eq!(lobby.clients.get(&guest).unwrap().name(), "Watcher");
    lobby.on_client_message(&sender, guest, ToServerPacket::SetName { name: "  \n\r  ".into() });
    assert!(!lobby.clients.get(&guest).unwrap().name().is_empty());
}

#[test]
fn names_survive_starting_a_game_and_returning_to_lobby_and_appear_in_host_data() {
    let (sender, mut rx) = connection();
    let mut lobby = Lobby::new();
    let host = lobby.join_client(&sender).unwrap().id;
    let _ = lobby.join_client(&sender).unwrap();
    let _ = lobby.join_client(&sender).unwrap();
    let spectator_id = lobby.join_client(&sender).unwrap().id;
    lobby.on_client_message(&sender, spectator_id, ToServerPacket::SetSpectator { spectator: true });
    lobby.set_player_name(spectator_id, "Watcher".into());
    lobby.settings.role_list = RoleList(vec![RoleOutline::new_exact(Role::Detective), RoleOutline::new_exact(Role::Detective), RoleOutline::new_exact(Role::Mafioso)]);
    lobby.settings.enabled_roles = Role::values().into_iter().collect();
    let LobbyClientMessageResult::StartGame(mut game) = lobby.on_client_message(&sender, host, ToServerPacket::StartGame)
        else { panic!("Expected game to start") };
    let pointer = SpectatorPointer::new(0);
    assert_eq!(pointer.deref(&game).unwrap().name, "Watcher");

    game.on_client_message(&sender, host, ToServerPacket::HostDataRequest);
    let mut host_name = None;
    while let Ok(packet) = rx.try_recv() {
        if let ToClientPacket::HostData { clients, .. } = packet {
            host_name = Some(clients.get(&spectator_id).unwrap().name.clone());
        }
    }
    assert_eq!(host_name.as_deref(), Some("Watcher"));

    // Spectator hosts use the same rename command and validation as player hosts.
    game.on_client_message(&sender, spectator_id, ToServerPacket::HostForceSetPlayerName { id: spectator_id, name: "Unauthorized".into() });
    assert_eq!(pointer.deref(&game).unwrap().name, "Watcher");
    game.on_client_message(&sender, host, ToServerPacket::SetPlayerHost { player_id: spectator_id });
    let desired = "  RenamedABCDEFGHIJKLMNOPQRSTUVWXYZ\n".to_string();
    game.on_client_message(&sender, spectator_id, ToServerPacket::HostForceSetPlayerName { id: spectator_id, name: desired.clone() });
    let expected = sanitize_name(desired, &[]);
    assert_eq!(pointer.deref(&game).unwrap().name, expected);

    let mafia_server::game::on_client_message::GameClientMessageResult::BackToLobby(mut returned) =
        game.on_client_message(&sender, host, ToServerPacket::HostForceBackToLobby)
        else { panic!("Expected return to lobby") };
    assert_eq!(returned.clients.get(&spectator_id).unwrap().name(), &expected);
    returned.on_client_message(&sender, spectator_id, ToServerPacket::SetSpectator { spectator: false });
    assert_eq!(returned.clients.get(&spectator_id).unwrap().name(), &expected);
}
