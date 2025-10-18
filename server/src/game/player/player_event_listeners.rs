use crate::game::{
    event::{
        on_midnight::{MidnightVariables, OnMidnight, OnMidnightPriority},
    },
    visit::Visit, Game
};

use super::PlayerReference;

impl PlayerReference {

    pub fn on_midnight(game: &mut Game, _event: &OnMidnight, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority){
        for player in PlayerReference::all_players(game){
            match priority {
                OnMidnightPriority::InitializeNight => {
                    player.set_night_grave_will(midnight_variables, player.alibi(game).to_owned());
                },
                OnMidnightPriority::FinalizeNight => {
                    player.push_night_messages_to_player(game, midnight_variables);
                }
                _ => {}
            }
            player.on_midnight_one_player(game, midnight_variables, priority);
        }
    }

    pub fn on_player_roleblocked(&self, game: &mut Game, midnight_variables: &mut MidnightVariables, player: PlayerReference, invisible: bool) {
        self.role_state(game).clone().on_player_roleblocked(game, midnight_variables, *self, player, invisible)
    }
    pub fn on_visit_wardblocked(&self, game: &mut Game, midnight_variables: &mut MidnightVariables, visit: Visit) {
        self.role_state(game).clone().on_visit_wardblocked(game, midnight_variables, *self, visit)
    }
}