use rand::seq::IndexedRandom;

use crate::game::{
    chat::ChatMessageVariant, components::{insider_group::InsiderGroupID, verdicts_today::VerdictsToday}, event::{on_any_death::OnAnyDeath, on_game_start::OnGameStart, on_remove_insider::OnRemoveInsider, on_role_switch::OnRoleSwitch}, player::PlayerReference, role::Role, Game
};

#[derive(Default, Debug, Clone)]
pub struct Cult {
    pub sacrifices: u8,

    pub player_executed: bool
}
impl Cult{
    pub fn on_game_start(game: &mut Game, _event: &OnGameStart, _fold: &mut (), _priority: ()){
        Self::ensure_apostle_exists(game);
    }
    fn ensure_apostle_exists(game: &mut Game){
        let mut cult_insiders: Vec<PlayerReference> = PlayerReference::all_players(game)
            .filter(|p|InsiderGroupID::Cult.contains_player(game, *p))
            .filter(|p|p.alive(game))
            .collect();
        
        let apostle_exists = cult_insiders.iter()
            .any(|p|matches!(p.role(game), Role::Apostle));
        
        if !apostle_exists && let Some(p) = cult_insiders.choose(&mut rand::rng()){
            p.set_new_role(game, Role::Apostle.default_state(), true);
        }

        cult_insiders.retain(|p|!matches!(p.role(game),Role::Apostle));
        
        let zealot_exists = cult_insiders.iter()
            .any(|p|matches!(p.role(game), Role::Zealot));

        if !zealot_exists && let Some(p) = cult_insiders.choose(&mut rand::rng()){
            p.set_new_role(game, Role::Zealot.default_state(), true);
        }
    }

    pub fn on_any_death(game: &mut Game, _event: &OnAnyDeath, _fold: &mut (), _priority: ()) {
        Cult::increment_sacrifices(game);
        Self::ensure_apostle_exists(game);
    }
    pub fn on_role_switch(game: &mut Game, _event: &OnRoleSwitch, _fold: &mut (), _priority: ()) {
        Self::ensure_apostle_exists(game);
    }
    pub fn on_remove_insider(game: &mut Game, _event: &OnRemoveInsider, _fold: &mut (), _priority: ()){
        Self::ensure_apostle_exists(game);
    }


    pub fn enough_sacrifices(game: &Game)->bool{
        game.cult.sacrifices >= 2
    }
    
    fn increment_sacrifices(game: &mut Game){
        Self::set_sacrifices(game, game.cult.sacrifices.saturating_add(1));
    }
    pub fn use_sacrifices(game: &mut Game){
        Self::set_sacrifices(game, game.cult.sacrifices.saturating_sub(2));
    }
    fn set_sacrifices(game: &mut Game, count: u8){
        game.cult.sacrifices = count;
        Self::send_sacrifice_count(game);
    }

    pub fn can_kill_tonight(game: &Game)->bool{
        VerdictsToday::player_last_executed(game).is_none()
    }

    pub fn send_sacrifice_count(game: &mut Game){
        for p in PlayerReference::all_players(game){
            if !InsiderGroupID::Cult.contains_player(game, p) {
                continue;
            }
            p.add_private_chat_message(game, ChatMessageVariant::CultSacrificeCount{count: game.cult.sacrifices});
        }
    }
}