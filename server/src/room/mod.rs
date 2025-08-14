
pub mod on_client_message;
pub mod name_validation;

use std::time::Duration;

use super::lobby::Lobby;

use crate::{
    game::Game, packet::{RoomPreviewData, RejectJoinReason, ToClientPacket}, websocket_connections::connection::ClientSender
};


pub type RoomClientID = u32;

#[enum_delegate::register]
pub trait RoomState {
    fn send_to_client_by_id(&self, room_client_id: RoomClientID, packet: ToClientPacket);
    fn join_client(&mut self, send: &ClientSender) -> Result<JoinRoomClientResult, RejectJoinReason>;
    fn initialize_client(&mut self, room_client_id: RoomClientID, send: &ClientSender);
    fn remove_client(&mut self, id: RoomClientID) -> RemoveRoomClientResult;
    fn remove_client_rejoinable(&mut self, id: RoomClientID) -> RemoveRoomClientResult;
    fn rejoin_client(&mut self, send: &ClientSender, room_client_id: RoomClientID) -> Result<JoinRoomClientResult, RejectJoinReason>;
    fn tick(&mut self, time_passed: Duration) -> RoomTickResult;
    fn get_preview_data(&self) -> RoomPreviewData;
    fn is_host(&self, room_client_id: RoomClientID)->bool;
}

#[expect(clippy::large_enum_variant, reason = "This is stored in a box anyway, and I don't want the rooms moving around in memory for no reason.")]
#[enum_delegate::implement(RoomState)]
pub enum Room {
    Lobby(Lobby),
    Game(Game),
}

#[must_use = "Send the accept join packet"]
pub struct JoinRoomClientResult {
    pub id: RoomClientID,
    pub in_game: bool,
    pub spectator: bool 
}

#[must_use = "You may need to close the room"]
pub enum RemoveRoomClientResult {
    Success,
    ClientNotInRoom,
    RoomShouldClose,
}

#[must_use = "You may need to close the room"]
pub struct RoomTickResult {
    pub close_room: bool,
}

impl Room {
    pub fn new() -> Room {
        Room::Lobby(Lobby::new())
    }
}
