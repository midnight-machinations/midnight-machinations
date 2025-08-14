pub mod chat_group;
pub mod chat_message;
pub mod chat_message_variant;
pub mod chat_controller;

use std::collections::VecDeque;

pub use chat_group::*;
pub use chat_message::*;
pub use chat_message_variant::*;
pub use chat_controller::*;

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
        PlayerComponent::new_component_box(player_count, |_|ChatPlayerComponent::new())
    }

    pub fn add_chat_message(game: &mut Game, player: PlayerReference, message: ChatMessage) {
        let chat_player = game.chat_messages.get_mut(player);
        let index = chat_player.messages.len();
        chat_player.messages.push(message.clone());
        chat_player.not_sent_messages.push_back((index, message.clone()));
    }
    pub fn chat_messages<'a>(game: &'a Game, player: PlayerReference) -> &'a Vec<ChatMessage> {
        &game.chat_messages.get(player).messages
    }
    pub fn not_sent_messages<'a>(game: &'a Game, player: PlayerReference) -> &'a VecDeque<(ChatMessageIndex, ChatMessage)> {
        &game.chat_messages.get(player).not_sent_messages
    }
    pub fn not_sent_messages_pop_front<'a>(game: &'a mut Game, player: PlayerReference) -> Option<(ChatMessageIndex, ChatMessage)> {
        game.chat_messages.get_mut(player).not_sent_messages.pop_front()
    }
    // #[expect(clippy::assigning_clones, reason = "Reference rules prevents this")]
    pub fn requeue_chat_messages(game: &mut Game, player: PlayerReference){
        let mut messages = Self::chat_messages(game, player)
            .clone()
            .into_iter()
            .enumerate()
            .collect();

        game.chat_messages.get_mut(player).not_sent_messages.append(&mut messages);
    }
}