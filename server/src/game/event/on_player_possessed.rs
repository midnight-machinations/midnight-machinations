use crate::game::{abilities_component::Abilities, components::mafia::Mafia, event::on_midnight::MidnightVariables, player::PlayerReference, Game};

pub struct OnPlayerPossessed{
    pub possessed: PlayerReference,
    pub possessed_into: PlayerReference,
}
impl OnPlayerPossessed{
    pub fn invoke(self, game: &mut Game, midnight_variables: &mut MidnightVariables){
        Abilities::on_player_possessed(game, &self, midnight_variables, ());
        Mafia::on_player_possessed(game, &self, midnight_variables, ());
    }
    pub fn new(possessed: PlayerReference, possessed_into: PlayerReference)->Self{
        Self { possessed, possessed_into }
    }
}