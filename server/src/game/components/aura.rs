use crate::game::{
    abilities_component::ability_id::AbilityID, event::on_midnight::MidnightVariables,
    player::PlayerReference, role::{arsonist::Arsonist, Role}, Game
};

pub struct Aura;
impl Aura{
    pub fn innocent(game: &Game, midnight_variables: &MidnightVariables, player: PlayerReference) -> bool {
        match player.role(game) {
            Role::Godfather => !player.night_blocked(midnight_variables),
            Role::Disguiser => !player.night_blocked(midnight_variables),
            Role::Pyrolisk => game.day_number() == 1,
            _ => false,
        }
    }
    pub fn suspicious(game: &Game, midnight_variables: &MidnightVariables, player: PlayerReference) -> bool {
        player.night_framed(midnight_variables) ||
        AbilityID::Role { role: Role::Drunk, player }.exists(game) ||
        Arsonist::has_suspicious_aura_douse(game, player)
    }
}