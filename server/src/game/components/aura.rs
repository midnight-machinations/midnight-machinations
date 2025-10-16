use crate::game::{
    abilities_component::ability_id::AbilityID, event::on_midnight::MidnightVariables,
    player::PlayerReference, role::{arsonist::Arsonist, Role}, Game
};

pub struct Aura;
impl Aura{
    pub fn innocent(game: &Game, midnight_variables: &MidnightVariables, player: PlayerReference) -> bool {
        ((AbilityID::Role { role: Role::Godfather, player }).exists(game) && !player.night_blocked(midnight_variables)) ||
        ((AbilityID::Role { role: Role::Disguiser, player }).exists(game) && !player.night_blocked(midnight_variables)) ||
        ((AbilityID::Role { role: Role::Pyrolisk, player }).exists(game) && game.day_number() == 1)
    }
    pub fn suspicious(game: &Game, midnight_variables: &MidnightVariables, player: PlayerReference) -> bool {
        player.night_framed(midnight_variables) ||
        AbilityID::Role { role: Role::Drunk, player }.exists(game) ||
        Arsonist::has_suspicious_aura_douse(game, player)
    }
}