use crate::game::{abilities::role_abilities::RoleAbility, abilities_component::{ability::Ability, ability_id::AbilityID, Abilities}, event::{on_midnight::{MidnightVariables, OnMidnight, OnMidnightPriority}, on_validated_ability_input_received::OnValidatedControllerInputReceived, on_whisper::{OnWhisper, WhisperFold, WhisperPriority}}, Game};

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
}
impl AbilityID{
    pub fn on_midnight(&self, game: &mut Game, _event: &OnMidnight, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority){
        match if let Some(ability) = self.get(game) {ability} else {return} {
            Ability::RoleAbility(RoleAbility(player, role_state)) => {
                role_state.clone().on_midnight(game, midnight_variables, *player, priority);
            }
        }
    }
    pub fn on_whisper(&self, game: &mut Game, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority){
        match if let Some(ability) = self.get(game) {ability} else {return} {
            Ability::RoleAbility(RoleAbility(player, role_state)) => {
                role_state.clone().on_whisper(game, *player, event, fold, priority);
            }
        }
    }
    pub fn on_validated_ability_input_received(&self, game: &mut Game, event: &OnValidatedControllerInputReceived, _fold: &mut (), _priority: ()) {
        match if let Some(ability) = self.get(game) {ability} else {return} {
            Ability::RoleAbility(RoleAbility(player, role_state)) => {
                role_state.clone().on_validated_ability_input_received(game, *player, event.actor_ref, event.input.clone())
            }
        }
    }
}