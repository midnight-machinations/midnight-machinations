use crate::game::{
    chat::{ChatGroup, ChatMessageVariant}, modifiers::ModifierID, player::PlayerReference, Game
};

use super::tags::Tags;

pub struct Enfranchise;
impl Enfranchise{
    pub fn enfranchise(game: &mut Game, player: PlayerReference){
        game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::MayorRevealed { player_index: player });

        Tags::add_tag(game, super::tags::TagSetID::Enfranchised, player);

        game.count_nomination_and_start_trial(
            game.modifier_settings().is_enabled(ModifierID::UnscheduledNominations)
        );
    }
    pub fn unenfranchise(game: &mut Game, player: PlayerReference){
        Tags::remove_tag(game, super::tags::TagSetID::Enfranchised, player);
    }
    pub fn enfranchised(game: &Game, player: PlayerReference)->bool{
        Tags::has_tag(game, super::tags::TagSetID::Enfranchised, player)
    }
    pub fn on_game_start(game: &mut Game){
        Tags::set_viewers(game, super::tags::TagSetID::Enfranchised, &PlayerReference::all_players(game).collect());
    }
}