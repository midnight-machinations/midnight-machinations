use crate::game::{
    abilities::role_abilities::RoleAbility,
    abilities_component::{ability::Ability, ability_id::AbilityID, Abilities},
    controllers::ControllerParametersMap,
    event::{
        before_phase_end::BeforePhaseEnd, on_ability_creation::{OnAbilityCreation, OnAbilityCreationFold, OnAbilityCreationPriority}, on_add_insider::OnAddInsider, on_any_death::OnAnyDeath, on_conceal_role::OnConcealRole, on_controller_selection_changed::OnControllerSelectionChanged, on_grave_added::OnGraveAdded, on_midnight::{MidnightVariables, OnMidnight, OnMidnightPriority}, on_phase_start::OnPhaseStart, on_remove_insider::OnRemoveInsider, on_validated_ability_input_received::OnValidatedControllerInputReceived, on_whisper::{OnWhisper, WhisperFold, WhisperPriority}
    }, Game
};

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
    pub fn on_grave_added(game: &mut Game, event: &OnGraveAdded, fold: &mut (), priority: ()){
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_grave_added(game, event, fold, priority)
        }
    }
    pub fn on_validated_ability_input_received(game: &mut Game, event: &OnValidatedControllerInputReceived, fold: &mut (), priority: ()) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_validated_ability_input_received(game, event, fold, priority)
        }
    }
    pub fn on_controller_selection_changed(game: &mut Game, event: &OnControllerSelectionChanged, fold: &mut (), priority: ()) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_controller_selection_changed(game, event, fold, priority)
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
    pub fn on_conceal_role(game: &mut Game, event: &OnConcealRole, fold: &mut (), priority: ()) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_conceal_role(game, event, fold, priority);
        }
    }
    pub fn on_add_insider(game: &mut Game, event: &OnAddInsider, fold: &mut (), priority: ()) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_add_insider(game, event, fold, priority);
        }
    }
    pub fn on_remove_insider(game: &mut Game, event: &OnRemoveInsider, fold: &mut (), priority: ()) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_remove_insider(game, event, fold, priority);
        }
    }
    pub fn on_any_death(game: &mut Game, event: &OnAnyDeath, fold: &mut (), priority: ()) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_any_death(game, event, fold, priority);
        }
    }
    pub fn on_ability_creation(game: &mut Game, event: &OnAbilityCreation, fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_ability_creation(game, event, fold, priority);
        }

        if priority == OnAbilityCreationPriority::SetAbility && !fold.cancelled{
            game.abilities.abilities.insert(event.id.clone(), fold.ability.clone());
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
        match if let Some(ability) = self.get(game).cloned() {ability} else {return} {
            Ability::RoleAbility(RoleAbility(player, ability)) => {
                ability.on_midnight(game, midnight_variables, player, priority);
            },
            Ability::Pitchfork(ability) => {
                ability.on_midnight(game, _event, midnight_variables, priority);
            },
            Ability::SyndicateGun(ability) => {
                ability.on_midnight(game, _event, midnight_variables, priority);
            }
        }
    }
    pub fn on_whisper(&self, game: &mut Game, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority){
        match if let Some(ability) = self.get(game).cloned() {ability} else {return} {
            Ability::RoleAbility(RoleAbility(player, role_state)) => {
                role_state.clone().on_whisper(game, player, event, fold, priority);
            },
            Ability::Pitchfork(_) => {},
            Ability::SyndicateGun(_) => {}
        }
    }
    pub fn on_grave_added(&self, game: &mut Game, event: &OnGraveAdded, _fold: &mut (), _priority: ()){
        match if let Some(ability) = self.get(game).cloned() {ability} else {return} {
            Ability::RoleAbility(RoleAbility(player, role_state)) => {
                role_state.on_grave_added(game, player, event.grave);
            },
            Ability::Pitchfork(_) => {},
            Ability::SyndicateGun(_) => {}
        }
    }
    pub fn on_validated_ability_input_received(&self, game: &mut Game, event: &OnValidatedControllerInputReceived, fold: &mut (), priority: ()) {
        match if let Some(ability) = self.get(game).cloned() {ability} else {return} {
            Ability::RoleAbility(RoleAbility(player, role_state)) => {
                role_state.on_validated_ability_input_received(game, player, event.actor_ref, event.input.clone())
            },
            Ability::Pitchfork(_) => {},
            Ability::SyndicateGun(ability) => {
                ability.on_validated_ability_input_received(game, event, fold, priority);
            }
        }
    }
    pub fn on_controller_selection_changed(&self, game: &mut Game, event: &OnControllerSelectionChanged, _fold: &mut (), _priority: ()){
        match if let Some(ability) = self.get(game).cloned() {ability} else {return} {
            Ability::RoleAbility(RoleAbility(player, role_state)) => {
                role_state.on_controller_selection_changed(game, player, event.id.clone())
            },
            Ability::Pitchfork(_) => {},
            Ability::SyndicateGun(_) => {}
        }
    }
    pub fn on_phase_start(&self, game: &mut Game, event: &OnPhaseStart, _fold: &mut (), _priority: ()) {
        match if let Some(ability) = self.get(game).cloned() {ability} else {return} {
            Ability::RoleAbility(RoleAbility(player, role_state)) => {
                role_state.on_phase_start(game, player, event.phase.phase())
            },
            Ability::Pitchfork(_) => {},
            Ability::SyndicateGun(_) => {}
        }
    }
    pub fn before_phase_end(&self, game: &mut Game, event: &BeforePhaseEnd, fold: &mut (), priority: ()) {
        match if let Some(ability) = self.get(game).cloned() {ability} else {return} {
            Ability::RoleAbility(_) => {},
            Ability::Pitchfork(pitchfork) => {
                pitchfork.before_phase_end(game, event, fold, priority);
            },
            Ability::SyndicateGun(_) => {}
        }
    }
    pub fn on_conceal_role(&self, game: &mut Game, event: &OnConcealRole, _fold: &mut (), _priority: ()) {
        match if let Some(ability) = self.get(game).cloned() {ability} else {return} {
            Ability::RoleAbility(RoleAbility(player, role_state)) => {
                let &OnConcealRole{player: event_player, concealed_player} = event;
                role_state.on_conceal_role(game, player, event_player, concealed_player)
            },
            Ability::Pitchfork(_) => {},
            Ability::SyndicateGun(_) => {}
        }
    }
    pub fn on_add_insider(&self, game: &mut Game, event: &OnAddInsider, fold: &mut (), priority: ()) {
        match if let Some(ability) = self.get(game).cloned() {ability} else {return} {
            Ability::RoleAbility(_) => {},
            Ability::Pitchfork(_) => {},
            Ability::SyndicateGun(ability) => {
                ability.on_add_insider(game, event, fold, priority);
            }
        }
    }
    pub fn on_remove_insider(&self, game: &mut Game, event: &OnRemoveInsider, fold: &mut (), priority: ()) {
        match if let Some(ability) = self.get(game).cloned() {ability} else {return} {
            Ability::RoleAbility(_) => {},
            Ability::Pitchfork(_) => {},
            Ability::SyndicateGun(ability) => {
                ability.on_remove_insider(game, event, fold, priority);
            }
        }
    }
    pub fn on_any_death(&self, game: &mut Game, event: &OnAnyDeath, fold: &mut (), priority: ()) {
        match if let Some(ability) = self.get(game).cloned() {ability} else {return} {
            Ability::RoleAbility(RoleAbility(player, role_state)) => {
                role_state.on_any_death(game, player, event.dead_player)
            },
            Ability::Pitchfork(_) => {},
            Ability::SyndicateGun(ability) => {
                ability.on_any_death(game, event, fold, priority);
            }
        }
    }
    pub fn on_ability_creation(&self, game: &mut Game, event: &OnAbilityCreation, fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        match if let Some(ability) = self.get(game).cloned() {ability} else {return} {
            Ability::RoleAbility(RoleAbility(player, role_state)) => {
                role_state.on_ability_creation(game, player, event, fold, priority)
            },
            Ability::Pitchfork(_) => {},
            Ability::SyndicateGun(_) => {}
        }
    }


    pub fn controller_parameters_map(&self, game: &Game) -> ControllerParametersMap {
        match if let Some(ability) = self.get(game).cloned() {ability} else {return ControllerParametersMap::default()} {
            Ability::RoleAbility(RoleAbility(player, role_state)) => {
                role_state.controller_parameters_map(game, player)
            },
            Ability::Pitchfork(ability) => {
                ability.controller_parameters_map(game)
            },
            Ability::SyndicateGun(ability) => {
                ability.controller_parameters_map(game)
            }
        }
    }
}