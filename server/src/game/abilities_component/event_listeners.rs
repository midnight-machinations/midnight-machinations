use crate::game::{abilities::role_abilities::RoleAbility, abilities_component::{ability::Ability, ability_id::AbilityID, Abilities}, controllers::ControllerParametersMap, event::{before_phase_end::BeforePhaseEnd, on_midnight::{MidnightVariables, OnMidnight, OnMidnightPriority}, on_phase_start::OnPhaseStart, on_validated_ability_input_received::OnValidatedControllerInputReceived, on_whisper::{OnWhisper, WhisperFold, WhisperPriority}}, Game};

impl Abilities{
    pub fn on_midnight(game: &mut Game, _event: &OnMidnight, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority){
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_midnight(game, _event, midnight_variables, priority)
        }
    }
    pub fn on_whisper(game: &mut Game, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_whisper(game, event, fold, priority)
        }
    }
    pub fn on_validated_ability_input_received(game: &mut Game, event: &OnValidatedControllerInputReceived, fold: &mut (), priority: ()) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_validated_ability_input_received(game, event, fold, priority)
        }
    }
    pub fn on_phase_start(game: &mut Game, event: &OnPhaseStart, _fold: &mut (), _priority: ()){
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_phase_start(game, event, _fold, _priority);
        }
    }
    pub fn before_phase_end(game: &mut Game, event: &BeforePhaseEnd, _fold: &mut (), _priority: ()){
        for (id, _ability) in game.abilities.abilities.clone() {
            id.before_phase_end(game, event, _fold, _priority);
        }
    }


    pub fn controller_parameters_map(game: &Game) -> ControllerParametersMap {
        ControllerParametersMap::combine(
            game.abilities.abilities
                .clone()
                .into_iter()
                .map(|a|a.0.controller_parameters_map(game))
        )
    }
}
impl AbilityID{
    pub fn on_midnight(&self, game: &mut Game, _event: &OnMidnight, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority){
        match if let Some(ability) = self.get(game) {ability} else {return} {
            Ability::RoleAbility(RoleAbility(player, role_state)) => {
                role_state.clone().on_midnight(game, midnight_variables, *player, priority);
            },
            Ability::Pitchfork(pitchfork) => {
                pitchfork.clone().on_midnight(game, _event, midnight_variables, priority);
            }
        }
    }
    pub fn on_whisper(&self, game: &mut Game, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority){
        match if let Some(ability) = self.get(game) {ability} else {return} {
            Ability::RoleAbility(RoleAbility(player, role_state)) => {
                role_state.clone().on_whisper(game, *player, event, fold, priority);
            },
            Ability::Pitchfork(pitchfork) => {
                
            }
        }
    }
    pub fn on_validated_ability_input_received(&self, game: &mut Game, event: &OnValidatedControllerInputReceived, _fold: &mut (), _priority: ()) {
        match if let Some(ability) = self.get(game) {ability} else {return} {
            Ability::RoleAbility(RoleAbility(player, role_state)) => {
                role_state.clone().on_validated_ability_input_received(game, *player, event.actor_ref, event.input.clone())
            },
            Ability::Pitchfork(pitchfork) => {
                // pitchfork.clone().on_validated_ability_input_received(game, event, fold, priority);
            }
        }
    }
    pub fn on_phase_start(&self, game: &mut Game, event: &OnPhaseStart, _fold: &mut (), _priority: ()) {
        match if let Some(ability) = self.get(game) {ability} else {return} {
            Ability::RoleAbility(RoleAbility(player, role_state)) => {
                role_state.clone().on_phase_start(game, *player, event.phase.phase())
            },
            Ability::Pitchfork(pitchfork) => {
                // pitchfork.clone().on_phase_start(game, event, fold, priority);
            }
        }
    }
    pub fn before_phase_end(&self, game: &mut Game, event: &BeforePhaseEnd, fold: &mut (), priority: ()) {
        match if let Some(ability) = self.get(game) {ability} else {return} {
            Ability::RoleAbility(RoleAbility(player, role_state)) => {
                role_state.clone().on_phase_start(game, *player, event.phase)
            },
            Ability::Pitchfork(pitchfork) => {
                pitchfork.clone().before_phase_end(game, event, fold, priority);
            }
        }
    }


    pub fn controller_parameters_map(&self, game: &Game) -> ControllerParametersMap {
        match if let Some(ability) = self.get(game) {ability} else {return ControllerParametersMap::default()} {
            Ability::RoleAbility(RoleAbility(player, role_state)) => {
                role_state.clone().controller_parameters_map(game, *player)
            },
            Ability::Pitchfork(pitchfork) => {
                pitchfork.clone().controller_parameters_map(game)
            }
        }
    }
}