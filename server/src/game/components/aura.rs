use crate::game::{
    abilities_component::ability_id::AbilityID, event::on_midnight::MidnightVariables,
    player::PlayerReference, role::{arsonist::Arsonist, Role}, Game
};

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
        AbilityID::Role { role: Role::Drunk, player }.get_ability(game).is_some() ||
        Arsonist::has_suspicious_aura_douse(game, player)
    }
}