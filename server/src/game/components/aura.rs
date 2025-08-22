use crate::game::{components::drunk_aura::DrunkAura, event::on_midnight::MidnightVariables, player::PlayerReference, role::arsonist::Arsonist, Game};

pub struct Aura;
impl Aura{
    pub fn innocent(game: &Game, player: PlayerReference) -> bool {
        player.role(game).has_innocent_aura(game)
    }
    pub fn suspicious(game: &Game, midnight_variables: &MidnightVariables, player: PlayerReference) -> bool {
        player.role(game).has_suspicious_aura(game) || 
        player.night_framed(midnight_variables) ||
        DrunkAura::has_drunk_aura(game, player) ||
        Arsonist::has_suspicious_aura_douse(game, player)
    }   
}