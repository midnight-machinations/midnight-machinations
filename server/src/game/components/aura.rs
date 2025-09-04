use crate::game::{components::drunk_aura::DrunkAura, event::on_midnight::MidnightVariables, player::PlayerReference, role::{arsonist::Arsonist, Role}, Game};

pub struct Aura;
impl Aura{
    pub fn innocent(game: &Game, player: PlayerReference) -> bool {
        match player.role(game) {
            Role::Godfather => true,
            Role::Disguiser => true,
            Role::Pyrolisk => {
                game.day_number() == 1
            },
            _ => false,
        }
    }
    pub fn suspicious(game: &Game, midnight_variables: &MidnightVariables, player: PlayerReference) -> bool {
        player.night_framed(midnight_variables) ||
        DrunkAura::has_drunk_aura(game, player) ||
        Arsonist::has_suspicious_aura_douse(game, player)
    }
}