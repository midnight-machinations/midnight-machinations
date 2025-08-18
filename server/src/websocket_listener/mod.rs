mod event;
mod client;
mod handle_message;

pub type RoomCode = usize;


use std::{collections::HashMap, net::SocketAddr, sync::{Arc, Mutex}, time::Duration};

use crate::{log, packet::{RejectJoinReason, ToClientPacket}, room::{JoinRoomClientResult, RemoveRoomClientResult, Room, RoomClientID, RoomState}, websocket_connections::connection::Connection};

use self::client::{Client, ClientLocation, ClientReference, GetRoomError};
use rand::random;



pub struct WebsocketListener {
    /// Clients that are currently connected, if a client isnt connected it isnt here
    ///
    /// What to do in different connection sscenarios:
    ///
    ///  Listener has client | Lobby has client | Behavior
    /// ---------------------|------------------|---------------------------
    ///  No                  | No               | Hooray!
    ///  No                  | Yes              | Reconnect (if possible)
    ///  Yes                 | No               | Disconnect listener client
    ///  Yes                 | Yes              | Hooray!
    clients: HashMap<SocketAddr, Client>,
    rooms: HashMap<RoomCode, Box<Room>>,
}
impl WebsocketListener{
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            clients: HashMap::new(),
        }
    }
    fn clients(&self) -> &HashMap<SocketAddr, Client> {
        &self.clients
    }
    fn rooms(&self) -> &HashMap<RoomCode, Box<Room>> {
        &self.rooms
    }
    fn rooms_mut(&mut self) -> &mut HashMap<RoomCode, Box<Room>> {
        &mut self.rooms
    }
    fn get_client<'a>(&'a self, address: &SocketAddr) -> Option<&'a Client> {
        self.clients.get(address)
    }
    fn get_client_mut<'a>(&'a mut self, address: &SocketAddr) -> Option<&'a mut Client> {
        self.clients.get_mut(address)
    }
    pub(super) fn get_room<'a>(&'a self, room_code: &RoomCode) -> Option<&'a Room> {
        self.rooms.get(room_code).map(|r| r.as_ref())
    }
    pub(super) fn get_room_mut<'a>(&'a mut self, room_code: &RoomCode) -> Option<&'a mut Room> {
        self.rooms.get_mut(room_code).map(|r| r.as_mut())
    }


    pub(super) fn create_client(&mut self, connection: &Connection) {

        if let Some(client_already_exists) = ClientReference::new(connection.address(), self){
            self.delete_client(&client_already_exists);
        }

        self.clients.insert(*connection.address(), Client::new(connection.clone()));
    }
    fn delete_client(&mut self, client: &ClientReference) {
        let Some(client) = self.clients.remove(&client.address(self).clone()) else {return};

        //This ToClientPacket is still useful in the *rare* case that the player is still connected when they're being forced to disconnect
        //A player can be forced to disconnect if a seperate connection is made with the same ip and port address
        client.send(ToClientPacket::ForcedDisconnect);


        let ClientLocation::InRoom { room_code, room_client_id } = client.location() else {return};
        let Some(room) = self.rooms.get_mut(room_code) else {return};

        match room.remove_client_rejoinable(*room_client_id) {
            RemoveRoomClientResult::Success |
            RemoveRoomClientResult::ClientNotInRoom => {},
            RemoveRoomClientResult::RoomShouldClose => self.delete_room(*room_code)
        }
    }


    fn set_client_in_room(&mut self, client: &ClientReference, room_code: RoomCode){

        let sender = &client.sender(self).clone();
        let Some(room) = self.get_room_mut(&room_code) else {
            client.send(self, ToClientPacket::RejectJoin { reason: RejectJoinReason::RoomDoesntExist });
            return
        };
        match room.join_client(sender) {
            Ok(JoinRoomClientResult { id: room_client_id, in_game, spectator }) => {
                sender.send(ToClientPacket::AcceptJoin { room_code, in_game, player_id: room_client_id, spectator });

                room.initialize_client(room_client_id, sender);

                client.set_location(self, ClientLocation::InRoom { room_code, room_client_id });
            }
            Err(reason) => {
                client.send(self, ToClientPacket::RejectJoin { reason });
            }
        }
    }
    fn set_client_in_room_reconnect(&mut self, client: ClientReference, room_code: RoomCode, room_client_id: RoomClientID){

        let sender = &client.sender(self).clone();
        let Some(room) = self.get_room_mut(&room_code) else {
            client.send(self, ToClientPacket::RejectJoin { reason: RejectJoinReason::RoomDoesntExist });
            return
        };
        match room.rejoin_client(sender, room_client_id) {
            Ok(JoinRoomClientResult { id: room_client_id, in_game, spectator }) => {
                sender.send(ToClientPacket::AcceptJoin { room_code, in_game, player_id: room_client_id, spectator });

                room.initialize_client(room_client_id, sender);

                client.set_location(self, ClientLocation::InRoom { room_code, room_client_id });
            }
            Err(reason) => {
                client.send(self, ToClientPacket::RejectJoin { reason });
            }
        }
    }
    fn set_client_outside_room(&mut self, client: &ClientReference, rejoinable: bool) {
        client.send(self, ToClientPacket::ForcedOutsideRoom);
        
        if let Ok((room, room_code, id)) = client.get_room_mut(self) {
            let result = if rejoinable {
                room.remove_client_rejoinable(id)
            }else{
                room.remove_client(id)
            };

            match result {
                RemoveRoomClientResult::Success |
                RemoveRoomClientResult::ClientNotInRoom => {}
                RemoveRoomClientResult::RoomShouldClose => self.delete_room(room_code),
            }
        }

        client.set_location(self, ClientLocation::OutsideRoom);
    }



    fn generate_roomcode(&self)->Option<RoomCode>{
        let start = random::<u16>() as usize;
        (start..=usize::MAX)
            .chain(0..start)
            .find(
                |code| !self.rooms.contains_key(code)
            )
    }
    pub(super) fn create_room(&mut self) -> Option<RoomCode>{
        let room_code = self.generate_roomcode()?;

        self.rooms.insert(room_code, Box::new(Room::new()));
        Some(room_code)
    }
    pub(super) fn delete_room(&mut self, room_code: RoomCode){
        self.rooms.remove(&room_code);

        for client in ClientReference::all_clients(self){
            if client.in_room(self, room_code) {
                client.send(self, ToClientPacket::ForcedOutsideRoom);
                client.set_location(self, ClientLocation::OutsideRoom);
            }
        }

        log!(important "Room"; "Closed {room_code}.");
    }

    
    pub fn start_tick(listener: Arc<Mutex<Self>>) {
        const DESIRED_FRAME_TIME: Duration = Duration::from_secs(1);

        tokio::spawn(async move {
            let mut frame_start_time = tokio::time::Instant::now();
            loop {
                let delta_time = frame_start_time.elapsed();
                frame_start_time = tokio::time::Instant::now();

                if let Ok(mut listener) = listener.lock() {
                    listener.tick(delta_time);                  
                } else { 
                    return;
                }

                tokio::time::sleep(DESIRED_FRAME_TIME.saturating_sub(frame_start_time.elapsed())).await;
            }
        });
    }


    fn validate_client(&self, addr: &SocketAddr)->Result<ClientReference,ValidateClientError>{
        let Some(client) = ClientReference::new(addr, self) else {return Err(ValidateClientError::ClientDoesntExist)};
        if let Err(GetRoomError::RoomDoesntExist) = client.get_room(self) {return Err(ValidateClientError::InRoomThatDoesntExist)};
        Ok(client)
    }
    
}


pub(super) enum ValidateClientError{
    ClientDoesntExist,
    InRoomThatDoesntExist
}
