use crate::{game::{components::player_component::PlayerComponent, event::on_midnight::MidnightVariables, player::PlayerReference, Game}, vec_set::VecSet};

pub type PitchforkItemComponent = PlayerComponent<PitchforkItem>;

#[derive(Default)]
pub struct PitchforkItem{
    has_item: bool  
}
impl PitchforkItemComponent{
    /// # Safety
    /// num_players must be correct
    pub unsafe fn new(num_players: u8) -> Self {
        unsafe {
            PlayerComponent::<PitchforkItem>::new_component_box(
                num_players,
                |_| PitchforkItem::default()
            )
        }
    }
    pub fn players_with_pitchfork(game: &Game, midnight_variables: &MidnightVariables) -> VecSet<PlayerReference>{
        PlayerReference::all_players(game)
            .filter(|p|game.pitchfork_item.get(*p).has_item)
            .filter(|p|!p.ability_deactivated_from_death(game))
            .filter(|p|!p.night_blocked(midnight_variables))
            .collect()
    }
    pub fn give_pitchfork(game: &mut Game, player: PlayerReference){
        game.pitchfork_item.get_mut(player).has_item = true;
    }
    pub fn remove_pitchfork(game: &mut Game, player: PlayerReference){
        game.pitchfork_item.get_mut(player).has_item = false;
    }
}