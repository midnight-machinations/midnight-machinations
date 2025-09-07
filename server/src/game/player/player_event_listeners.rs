use crate::game::{
    components::graves::grave_reference::GraveReference, controllers::{ControllerID, ControllerInput},
    event::{
        on_midnight::{MidnightVariables, OnMidnight, OnMidnightPriority},
    },
    role::RoleState, visit::Visit, Game
};

use super::PlayerReference;

impl PlayerReference {

    pub fn on_midnight(game: &mut Game, _event: &OnMidnight, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority){
        for player in PlayerReference::all_players(game){
            match priority {
                OnMidnightPriority::InitializeNight => {
                    player.set_night_grave_will(midnight_variables, player.alibi(game).to_owned());
                    let visits = player.convert_selection_to_visits(game);
                    player.set_night_visits(midnight_variables, visits.clone());
                },
                OnMidnightPriority::FinalizeNight => {
                    player.push_night_messages_to_player(game, midnight_variables);
                }
                _ => {}
            }
            player.on_midnight_one_player(game, midnight_variables, priority);
        }
    }



    pub fn on_controller_selection_changed(&self, game: &mut Game, id: ControllerID){
        self.role_state(game).clone().on_controller_selection_changed(game, *self, id)
    }
    pub fn on_ability_input_received(&self, game: &mut Game, input_player: PlayerReference, input: ControllerInput) {
        self.role_state(game).clone().on_ability_input_received(game, *self, input_player, input)
    }
    pub fn on_game_start(&self, game: &mut Game){
        self.role_state(game).clone().on_game_start(game, *self)
    }
    pub fn on_game_ending(&self, game: &mut Game){
        self.role_state(game).clone().on_game_ending(game, *self)
    }
    pub fn on_grave_added(&self, game: &mut Game, grave: GraveReference){
        self.role_state(game).clone().on_grave_added(game, *self, grave)
    }
    pub fn on_role_switch(&self, game: &mut Game, player: PlayerReference, old: RoleState, new: RoleState,){
        self.role_state(game).clone().on_role_switch(game, *self, player, old, new);
    }
    pub fn before_role_switch(&self, game: &mut Game, player: PlayerReference, old: RoleState, new: RoleState,){
        self.role_state(game).clone().before_role_switch(game, *self, player, old, new);
    }
    pub fn before_initial_role_creation(&self, game: &mut Game){
        self.role_state(game).clone().before_initial_role_creation(game, *self)
    }
    pub fn on_player_roleblocked(&self, game: &mut Game, midnight_variables: &mut MidnightVariables, player: PlayerReference, invisible: bool) {
        self.role_state(game).clone().on_player_roleblocked(game, midnight_variables, *self, player, invisible)
    }
    pub fn on_visit_wardblocked(&self, game: &mut Game, midnight_variables: &mut MidnightVariables, visit: Visit) {
        self.role_state(game).clone().on_visit_wardblocked(game, midnight_variables, *self, visit)
    }
}