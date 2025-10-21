pub mod chat_group;
pub mod chat_message;
pub mod chat_message_variant;
pub mod chat_controller;
use std::collections::VecDeque;
pub use chat_group::*;
pub use chat_message::*;
pub use chat_message_variant::*;
use crate::game::prelude::*;

use crate::game::{components::player_component::PlayerComponent, player::PlayerReference, Game};


pub type ChatComponent = PlayerComponent<ChatPlayerComponent>;
pub type ChatMessageIndex = usize;
#[derive(Default)]
pub struct ChatPlayerComponent{
    messages: Vec<ChatMessage>,
    not_sent_messages: VecDeque<(ChatMessageIndex, ChatMessage)>
}
impl ChatPlayerComponent{
    fn new()->Self{
        Self::default()
    }
}
impl ChatComponent{
    /// # Safety
    /// player_count is correct
    pub unsafe fn new(player_count: u8)->Self{
        unsafe { PlayerComponent::new_component_box(player_count, |_|ChatPlayerComponent::new()) }
    }

    pub fn add_chat_message(game: &mut Game, player: PlayerReference, message: ChatMessage) {
        let chat_player = game.chat_messages.get_mut(player);
        let index = chat_player.messages.len();
        chat_player.messages.push(message.clone());
        chat_player.not_sent_messages.push_back((index, message.clone()));
    }
    pub fn chat_messages(game: &Game, player: PlayerReference) -> &Vec<ChatMessage> {
        &game.chat_messages.get(player).messages
    }
    pub fn get_message(game: &Game, player: PlayerReference, message: ChatMessageIndex)->Option<&ChatMessage>{
        Self::chat_messages(game, player).get(message)
    }
    pub fn not_sent_messages(game: &Game, player: PlayerReference) -> &VecDeque<(ChatMessageIndex, ChatMessage)> {
        &game.chat_messages.get(player).not_sent_messages
    }
    pub fn not_sent_messages_pop_front(game: &mut Game, player: PlayerReference) -> Option<(ChatMessageIndex, ChatMessage)> {
        game.chat_messages.get_mut(player).not_sent_messages.pop_front()
    }
    pub fn requeue_chat_messages(game: &mut Game, player: PlayerReference){
        let mut messages = Self::chat_messages(game, player)
            .clone()
            .into_iter()
            .enumerate()
            .collect();

        game.chat_messages.get_mut(player).not_sent_messages.append(&mut messages);
    }



    
    pub fn on_whisper(game: &mut Game, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority) {
        match priority {
            WhisperPriority::Cancel => {
                if 
                    game.current_phase().phase() == PhaseType::Night || 
                    !event.receiver.alive(game) ||
                    !event.sender.alive(game) ||
                    event.receiver == event.sender || 
                    !event.sender.get_current_send_chat_groups(game).contains(&ChatGroup::All) ||
                    event.message.replace(['\n', '\r'], "").trim().is_empty()
                {
                    fold.cancelled = true;
                    fold.hide_broadcast = true;
                }
            },
            WhisperPriority::Broadcast => {
                if !fold.hide_broadcast {
                    game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::BroadcastWhisper {
                        whisperer: event.sender,
                        whisperee: event.receiver
                    });
                }
            },
            WhisperPriority::Send => {
                if fold.cancelled {
                    event.sender.add_private_chat_message(game, ChatMessageVariant::InvalidWhisper);
                } else {
                    let message = ChatMessageVariant::Whisper { 
                        from_player_index: event.sender, 
                        to_player_index: event.receiver, 
                        text: event.message.clone()
                    };

                    event.sender.add_private_chat_message(game, message.clone());
                    event.receiver.add_private_chat_message(game, message);
                }
            },
        }
    }
}