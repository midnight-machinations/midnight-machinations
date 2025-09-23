use rand::seq::IndexedRandom;

use crate::game::{
    components::{insider_group::InsiderGroupID, verdicts_today::VerdictsToday}, event::{on_any_death::OnAnyDeath, on_game_start::OnGameStart}, player::PlayerReference, role::Role, Game
};

#[derive(Default, Debug, Clone)]
pub struct Cult {
    pub sacrifices: u8,

    pub player_executed: bool
}
impl Cult{
    pub fn on_game_start(game: &mut Game, _event: &OnGameStart, _fold: &mut (), _priority: ()){
        let mut cult_insiders: Vec<PlayerReference> = PlayerReference::all_players(game)
            .filter(|p|InsiderGroupID::Cult.contains_player(game, *p))
            .collect();
        
        let apostle_exists = cult_insiders.iter()
            .any(|p|matches!(p.role(game), Role::Apostle));
        
        if !apostle_exists && let Some(p) = cult_insiders.choose(&mut rand::rng()){
            p.set_role(game, Role::Apostle.default_state());
        }

        cult_insiders.retain(|p|!matches!(p.role(game),Role::Apostle));
        
        let zealot_exists = cult_insiders.iter()
            .any(|p|matches!(p.role(game), Role::Zealot));

        if !zealot_exists && let Some(p) = cult_insiders.choose(&mut rand::rng()){
            p.set_role(game, Role::Zealot.default_state());
        }
    }

    pub fn on_any_death(game: &mut Game, _event: &OnAnyDeath, _fold: &mut (), _priority: ()) {
        Cult::increment_sacrifices(game);
    }

    fn increment_sacrifices(game: &mut Game){
        game.cult.sacrifices = game.cult.sacrifices.saturating_add(1);
    }
    pub fn enough_sacrifices(game: &Game)->bool{
        game.cult.sacrifices >= 2
    }
    pub fn use_sacrifices(game: &mut Game){
        game.cult.sacrifices = game.cult.sacrifices.saturating_sub(2);
    }

    pub fn can_kill_tonight(game: &Game)->bool{
        VerdictsToday::player_last_executed(game).is_none()
    }
}