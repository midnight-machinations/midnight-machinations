use crate::game::{
    chat::{ChatGroup, ChatMessageVariant}, components::player_component::PlayerComponent,
    event::on_game_start::OnGameStart, modifiers::ModifierID, player::PlayerReference, Game
};

use super::tags::Tags;

pub type EnfranchiseComponent = PlayerComponent<Option<EnfranchisePower>>;
pub struct EnfranchisePower{
    additional_votes: u8
} 
impl EnfranchiseComponent{
    /// # Safety
    /// player_count is correct
    pub unsafe fn new(num_players: u8) -> Self {
        unsafe {
            PlayerComponent::<Option<EnfranchisePower>>::new_component_box(
                num_players,
                |_| None
            )
        }
    }
    pub fn enfranchise(game: &mut Game, player: PlayerReference, additional_votes: u8){
        game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::PlayerEnfranchised { player_index: player });

        Tags::add_tag(game, super::tags::TagSetID::Enfranchised, player);
        *game.enfranchise.get_mut(player) = Some(EnfranchisePower{additional_votes});

        game.count_nomination_and_start_trial(
            game.modifier_settings().is_enabled(ModifierID::UnscheduledNominations)
        );
    }
    pub fn unenfranchise(game: &mut Game, player: PlayerReference){
        Tags::remove_tag(game, super::tags::TagSetID::Enfranchised, player);
        *game.enfranchise.get_mut(player) = None;
    }
    pub fn enfranchised(game: &Game, player: PlayerReference)->bool{
        game.enfranchise.get(player).is_some()
    }
    fn enfranchise_power(game: &Game, player: PlayerReference)->Option<u8>{
        game.enfranchise.get(player).as_ref().map(|o| o.additional_votes)
    }
    pub fn voting_power(game: &Game, player: PlayerReference)->u8{
        EnfranchiseComponent::enfranchise_power(game, player).unwrap_or(1)
    }
    pub fn on_game_start(game: &mut Game, _event: &OnGameStart, _fold: &mut (), _priority: ()){
        Tags::set_viewers(game, super::tags::TagSetID::Enfranchised, &PlayerReference::all_players(game).collect());
    }
}