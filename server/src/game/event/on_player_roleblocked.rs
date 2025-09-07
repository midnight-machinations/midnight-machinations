use crate::game::{ 
    abilities::syndicate_gun::SyndicateGun, chat::ChatMessageVariant, components::mafia::Mafia, player::PlayerReference, Game
};

use super::on_midnight::MidnightVariables;

#[must_use = "Event must be invoked"]
pub struct OnPlayerRoleblocked{
    player: PlayerReference,
    invisible: bool,
}
impl OnPlayerRoleblocked{
    pub fn new(player: PlayerReference, invisible: bool) -> Self{
        Self{player, invisible}
    }
    pub fn invoke(self, game: &mut Game, midnight_variables: &mut MidnightVariables){
        self.player.set_night_blocked(midnight_variables, true);
        if !self.invisible {
            self.player.push_night_message(midnight_variables,
                ChatMessageVariant::RoleBlocked
            );
        }
        
        for player_ref in PlayerReference::all_players(game){
            player_ref.on_player_roleblocked(game, midnight_variables, self.player, self.invisible);
        }
        Mafia::on_player_roleblocked(game, midnight_variables, self.player);
        SyndicateGun::on_player_roleblocked(game, midnight_variables, self.player);
    }
}