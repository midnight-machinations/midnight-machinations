use crate::game::{
    components::graves::grave_reference::GraveReference, controllers::{ControllerID, ControllerInput}, event::{
        on_midnight::{MidnightVariables, OnMidnight, OnMidnightPriority}, on_phase_start::OnPhaseStart, on_validated_ability_input_received::OnValidatedControllerInputReceived, on_whisper::{OnWhisper, WhisperFold, WhisperPriority}}, role::RoleState, visit::Visit, Game
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
        // Handle ability controller changes first
        if let ControllerID::Ability { ability, .. } = &id {
            if let Some(mut ability_state) = self.get_ability(game, *ability).cloned() {
                ability_state.on_controller_selection_changed(game, *self, id.clone());
                self.set_ability_state(game, *ability, ability_state);
            }
        }
        
        // Handle role controller changes
        self.role_state(game).clone().on_controller_selection_changed(game, *self, id);
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
    pub fn on_any_death(&self, game: &mut Game, dead_player_ref: PlayerReference){
        // Handle role death event
        self.role_state(game).clone().on_any_death(game, *self, dead_player_ref);
        
        // Handle ability death events
        let ability_ids: Vec<_> = self.abilities(game).keys().copied().collect();
        for ability_id in ability_ids {
            if let Some(mut ability_state) = self.get_ability(game, ability_id).cloned() {
                ability_state.on_any_death(game, *self, dead_player_ref);
                self.set_ability_state(game, ability_id, ability_state);
            }
        }
        
        // Remove abilities that should be removed on death
        if dead_player_ref == *self {
            let abilities_to_remove: Vec<_> = self.abilities(game)
                .iter()
                .filter(|(_, ability_state)| ability_state.removed_on_death())
                .map(|(ability_id, _)| *ability_id)
                .collect();
            
            for ability_id in abilities_to_remove {
                self.remove_ability(game, ability_id);
            }
        }
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
        // Handle role roleblock
        self.role_state(game).clone().on_player_roleblocked(game, midnight_variables, *self, player, invisible);
        
        // Handle ability roleblock
        let ability_ids: Vec<_> = self.abilities(game).keys().copied().collect();
        for ability_id in ability_ids {
            if let Some(mut ability_state) = self.get_ability(game, ability_id).cloned() {
                ability_state.on_player_roleblocked(game, midnight_variables, *self, player, invisible);
                self.set_ability_state(game, ability_id, ability_state);
            }
        }
    }
    pub fn on_visit_wardblocked(&self, game: &mut Game, midnight_variables: &mut MidnightVariables, visit: Visit) {
        // Handle role wardblock
        self.role_state(game).clone().on_visit_wardblocked(game, midnight_variables, *self, visit);
        
        // Handle ability wardblock
        let ability_ids: Vec<_> = self.abilities(game).keys().copied().collect();
        for ability_id in ability_ids {
            if let Some(mut ability_state) = self.get_ability(game, ability_id).cloned() {
                ability_state.on_visit_wardblocked(game, midnight_variables, *self, visit);
                self.set_ability_state(game, ability_id, ability_state);
            }
        }
    }
    pub fn on_whisper(game: &mut Game, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority) {
        for player in PlayerReference::all_players(game){
            player.role_state(game).clone().on_whisper(game, player, event, fold, priority);
        }
    }

    pub fn on_validated_ability_input_received(game: &mut Game, event: &OnValidatedControllerInputReceived, _fold: &mut (), _priority: ()) {
        for player_ref in PlayerReference::all_players(game){
            player_ref.one_player_on_validated_ability_input_received(game, event.actor_ref, event.input.clone())
        }
    }
    pub fn one_player_on_validated_ability_input_received(&self, game: &mut Game, input_player: PlayerReference, input: ControllerInput) {
        // Handle ability input for ability controllers first
        if let ControllerID::Ability { ability, .. } = &input.id {
            if let Some(mut ability_state) = self.get_ability(game, *ability).cloned() {
                ability_state.on_ability_input_received(game, *self, input_player, input.clone());
                self.set_ability_state(game, *ability, ability_state);
            }
            return;
        }
        
        // Handle role input
        if let ControllerID::Role { .. } = &input.id {
            // Let the role handle it
            self.role_state(game).clone().on_validated_ability_input_received(game, *self, input_player, input.clone());
        }
    }

    pub fn on_phase_start(game: &mut Game, event: &OnPhaseStart, _fold: &mut (), _priority: ()){
        for player_ref in PlayerReference::all_players(game){
            player_ref.one_on_phase_start(game, event, _fold, _priority);
        }
    }
    pub fn one_on_phase_start(&self, game: &mut Game, event: &OnPhaseStart, _fold: &mut (), _priority: ()){
        // Handle role phase start
        self.role_state(game).clone().on_phase_start(game, *self, event.phase.phase());
        
        // Handle ability phase starts
        let ability_ids: Vec<_> = self.abilities(game).keys().copied().collect();
        for ability_id in ability_ids {
            if let Some(mut ability_state) = self.get_ability(game, ability_id).cloned() {
                ability_state.on_phase_start(game, *self, event.phase.phase());
                self.set_ability_state(game, ability_id, ability_state);
            }
        }
    }
}