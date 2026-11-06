use mafia_server::{
    game::role::Role,
    lobby::Lobby,
    packet::{ToClientPacket, ToServerPacket},
    room::RoomState,
    websocket_connections::connection::{ClientSender, Connection},
};

fn connection() -> (ClientSender, tokio::sync::mpsc::UnboundedReceiver<ToClientPacket>) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    (Connection::new(tx, "127.0.0.1:0".parse().unwrap()).sender(), rx)
}

#[test]
fn only_the_host_can_change_enabled_roles() {
    let (sender, _rx) = connection();
    let mut lobby = Lobby::new();
    let host = lobby.join_client(&sender).unwrap().id;
    let guest = lobby.join_client(&sender).unwrap().id;
    assert!(lobby.is_host(host) && !lobby.is_host(guest));
    let before = lobby.settings.enabled_roles.clone();

    lobby.on_client_message(&sender, guest, ToServerPacket::SetEnabledRoles { roles: vec![Role::Detective] });
    assert_eq!(lobby.settings.enabled_roles, before);

    lobby.on_client_message(&sender, host, ToServerPacket::SetEnabledRoles { roles: vec![Role::Detective] });
    assert_eq!(lobby.settings.enabled_roles.iter().copied().collect::<Vec<_>>(), vec![Role::Detective]);
}
