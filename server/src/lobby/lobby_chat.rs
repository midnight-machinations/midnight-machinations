use std::collections::VecDeque;

use crate::{game::{chat::ChatMessage, game_client::{GameClient, GameClientLocation}, Game}, lobby::Lobby, packet::ToClientPacket, room::{RoomClientID, RoomState}, vec_map::VecMap};


pub struct LobbyChatComponent {
    pub chat: VecMap<RoomClientID, ChatClientComponent>,
}
impl LobbyChatComponent {
    pub fn get(&self, room_client_id: RoomClientID) -> Option<&ChatClientComponent> {
        self.chat.get(&room_client_id)
    }

    pub fn get_mut_or_insert(&mut self, room_client_id: RoomClientID) -> &mut ChatClientComponent {
        if self.chat.contains(&room_client_id) {
            #[expect(clippy::unwrap_used, reason = "We just checked it exists")]
            return self.chat.get_mut(&room_client_id).unwrap();
        }

        self.chat.insert(room_client_id, ChatClientComponent::default());
        #[expect(clippy::unwrap_used, reason = "We just added it")]
        self.chat.get_mut(&room_client_id).unwrap()
    }

    pub fn get_mut(&mut self, room_client_id: RoomClientID) -> Option<&mut ChatClientComponent> {
        self.chat.get_mut(&room_client_id)
    }
}
pub type ChatMessageIndex = usize;

#[derive(Default)]
pub struct ChatClientComponent{
    messages: Vec<ChatMessage>,
    not_sent_messages: VecDeque<(ChatMessageIndex, ChatMessage)>
}
impl LobbyChatComponent{
    pub fn new()->Self{
        Self {
            chat: VecMap::new(),
        }
    }

    pub fn from_game(game: &Game) -> Self {
        let mut out = LobbyChatComponent::new();

        let spectator_chat_messages: Vec<ChatMessage> = game.spectator_chat_messages.clone().iter()
            .map(|msg| ChatMessage { variant: msg.clone(), chat_group: None })
            .collect();

        for (room_client_id, GameClient { client_location, ..}) in game.clients.iter() {
            match client_location {
                GameClientLocation::Player(player_ref) => {
                    let player_chat_component = game.chat_messages.get(*player_ref);

                    out.chat.insert(*room_client_id, ChatClientComponent {
                        messages: player_chat_component.messages.clone(),
                        not_sent_messages: player_chat_component.not_sent_messages.clone()
                    });
                },
                GameClientLocation::Spectator(spectator_index) => {
                    out.chat.insert(*room_client_id, ChatClientComponent {
                        messages: spectator_chat_messages.clone(),
                        not_sent_messages: spectator_index.queued_chat_messages(game)
                            .iter()
                            .map(|(idx, msg)| (*idx, ChatMessage { variant: msg.clone(), chat_group: None }))
                            .collect(),
                    });
                },
            };
        }

        out
    }

    pub fn send_queued_messages(lobby: &mut Lobby) {
        for client in lobby.clients.clone().keys() {
            LobbyChatComponent::send_queued_messages_to_client(lobby, *client);
        }
    }

    pub fn send_queued_messages_to_client(lobby: &mut Lobby, client: RoomClientID) {
        let Some(unsent_messages) = lobby.chat.not_sent_messages(client).cloned() else { return; };

        if unsent_messages.is_empty() {
            return;
        }
        
        let mut chat_messages_out = vec![];

        // Send in chunks
        for _ in 0..5 {
            let msg_option = lobby.chat.not_sent_messages(client).and_then(|q| q.front());
            if let Some(msg) = msg_option{
                chat_messages_out.push(msg.clone());
                lobby.chat.not_sent_messages_pop_front(client);
            } else {break}
        }
        
        lobby.send_to_client_by_id(client, ToClientPacket::AddChatMessages {
            chat_messages: chat_messages_out.into_iter().collect()
        });
        

        LobbyChatComponent::send_queued_messages_to_client(lobby, client);
    }

    pub fn add_chat_message(&mut self, client: RoomClientID, message: ChatMessage) {
        let chat_player = self.get_mut_or_insert(client);
        let index = chat_player.messages.len();
        chat_player.messages.push(message.clone());
        chat_player.not_sent_messages.push_back((index, message.clone()));
    }
    pub fn chat_messages(&self, client: RoomClientID) -> Option<&Vec<ChatMessage>> {
        self.get(client).map(|c| &c.messages)
    }
    pub fn get_message(&self, client: RoomClientID, message: ChatMessageIndex)->Option<&ChatMessage>{
        Self::chat_messages(self, client).and_then(|messages| messages.get(message))
    }
    pub fn not_sent_messages(&self, client: RoomClientID) -> Option<&VecDeque<(ChatMessageIndex, ChatMessage)>> {
        self.get(client).map(|c| &c.not_sent_messages)
    }
    pub fn not_sent_messages_pop_front(&mut self, client: RoomClientID) -> Option<(ChatMessageIndex, ChatMessage)> {
        if let Some(client_chat) = self.get_mut(client) {
            client_chat.not_sent_messages.pop_front()
        } else {
            None
        }
    }
    pub fn requeue_chat_messages(&mut self, client: RoomClientID){
        let Some(messages) = Self::chat_messages(self, client) else { return };

        let mut messages = messages
            .clone()
            .into_iter()
            .enumerate()
            .collect();

        if let Some(client) = self.get_mut(client) {
            client.not_sent_messages.append(&mut messages);
        }
    }
}