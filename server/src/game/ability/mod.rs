use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::game::player::PlayerReference;
use crate::game::visit::Visit;
use crate::game::Game;
use crate::game::controllers::{ControllerParametersMap, ControllerID, ControllerInput};
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::phase::PhaseType;
use crate::game::components::graves::grave_reference::GraveReference;

pub mod vigilante_gun;

pub trait AbilityState: Clone + std::fmt::Debug + Default + Serialize {
    /// Called during midnight events at the specified priority
    fn on_midnight(&mut self, _game: &mut Game, _midnight_variables: &mut MidnightVariables, _actor_ref: PlayerReference, _priority: OnMidnightPriority) {
        // Default implementation does nothing
    }
    
    /// Returns controller parameters for this ability
    fn controller_parameters_map(&self, _game: &Game, _actor_ref: PlayerReference) -> ControllerParametersMap {
        // Default implementation returns empty controller map
        ControllerParametersMap::default()
    }
    
    /// Called when controller selection changes
    fn on_controller_selection_changed(&mut self, _game: &mut Game, _actor_ref: PlayerReference, _id: ControllerID) {
        // Default implementation does nothing
    }
    
    /// Called when ability input is received
    fn on_ability_input_received(&mut self, _game: &mut Game, _actor_ref: PlayerReference, _input_player: PlayerReference, _ability_input: ControllerInput) {
        // Default implementation does nothing
    }
    
    /// Converts controller selection to visits
    fn convert_selection_to_visits(&self, _game: &Game, _actor_ref: PlayerReference) -> Vec<Visit> {
        // Default implementation returns no visits
        Vec::new()
    }
    
    /// Called when phases start
    fn on_phase_start(&mut self, _game: &mut Game, _actor_ref: PlayerReference, _phase: PhaseType) {
        // Default implementation does nothing
    }
    
    /// Called when the ability is first given to a player
    fn on_ability_added(&mut self, _game: &mut Game, _actor_ref: PlayerReference) {
        // Default implementation does nothing
    }
    
    /// Called when the ability is removed from a player
    fn on_ability_removed(&mut self, _game: &mut Game, _actor_ref: PlayerReference) {
        // Default implementation does nothing
    }
    
    /// Called when player is roleblocked
    fn on_player_roleblocked(&mut self, _game: &mut Game, _midnight_variables: &mut MidnightVariables, _actor_ref: PlayerReference, _player: PlayerReference, _invisible: bool) {
        // Default implementation does nothing
    }
    
    /// Called when a visit is wardblocked
    fn on_visit_wardblocked(&mut self, _game: &mut Game, _midnight_variables: &mut MidnightVariables, _actor_ref: PlayerReference, _visit: Visit) {
        // Default implementation does nothing
    }
    
    /// Called when any player dies
    fn on_any_death(&mut self, _game: &mut Game, _actor_ref: PlayerReference, _dead_player_ref: PlayerReference) {
        // Default implementation does nothing
    }
    
    /// Called when a grave is added
    fn on_grave_added(&mut self, _game: &mut Game, _actor_ref: PlayerReference, _grave: GraveReference) {
        // Default implementation does nothing
    }
    
    /// Called when the game starts
    fn on_game_start(&mut self, _game: &mut Game, _actor_ref: PlayerReference) {
        // Default implementation does nothing
    }
    
    /// Called when the game is ending
    fn on_game_ending(&mut self, _game: &mut Game, _actor_ref: PlayerReference) {
        // Default implementation does nothing
    }
    
    /// Returns true if this ability should be removed when the player dies
    fn removed_on_death(&self) -> bool {
        true
    }
    
    /// Returns true if this ability is currently active
    fn is_active(&self) -> bool {
        true
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum AbilityID {
    VigilanteGun,
}

impl AbilityID {
    pub fn default_state(&self) -> AbilityStateEnum {
        match self {
            Self::VigilanteGun => AbilityStateEnum::VigilanteGun(vigilante_gun::VigilanteGun::default()),
        }
    }
    
    pub fn new_state(&self, game: &Game) -> AbilityStateEnum {
        match self {
            Self::VigilanteGun => AbilityStateEnum::VigilanteGun(vigilante_gun::VigilanteGun::new_state(game)),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AbilityStateEnum {
    VigilanteGun(vigilante_gun::VigilanteGun),
}

impl AbilityStateEnum {
    pub fn ability_id(&self) -> AbilityID {
        match self {
            Self::VigilanteGun(_) => AbilityID::VigilanteGun,
        }
    }
    
    pub fn on_midnight(&mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        match self {
            Self::VigilanteGun(ability) => ability.on_midnight(game, midnight_variables, actor_ref, priority),
        }
    }
    
    pub fn controller_parameters_map(&self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        match self {
            Self::VigilanteGun(ability) => ability.controller_parameters_map(game, actor_ref),
        }
    }
    
    pub fn on_controller_selection_changed(&mut self, game: &mut Game, actor_ref: PlayerReference, id: ControllerID) {
        match self {
            Self::VigilanteGun(ability) => ability.on_controller_selection_changed(game, actor_ref, id),
        }
    }
    
    pub fn on_ability_input_received(&mut self, game: &mut Game, actor_ref: PlayerReference, input_player: PlayerReference, ability_input: ControllerInput) {
        match self {
            Self::VigilanteGun(ability) => ability.on_ability_input_received(game, actor_ref, input_player, ability_input),
        }
    }
    
    pub fn convert_selection_to_visits(&self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        match self {
            Self::VigilanteGun(ability) => ability.convert_selection_to_visits(game, actor_ref),
        }
    }
    
    pub fn on_phase_start(&mut self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType) {
        match self {
            Self::VigilanteGun(ability) => ability.on_phase_start(game, actor_ref, phase),
        }
    }
    
    pub fn on_ability_added(&mut self, game: &mut Game, actor_ref: PlayerReference) {
        match self {
            Self::VigilanteGun(ability) => ability.on_ability_added(game, actor_ref),
        }
    }
    
    pub fn on_ability_removed(&mut self, game: &mut Game, actor_ref: PlayerReference) {
        match self {
            Self::VigilanteGun(ability) => ability.on_ability_removed(game, actor_ref),
        }
    }
    
    pub fn on_player_roleblocked(&mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, player: PlayerReference, invisible: bool) {
        match self {
            Self::VigilanteGun(ability) => ability.on_player_roleblocked(game, midnight_variables, actor_ref, player, invisible),
        }
    }
    
    pub fn on_visit_wardblocked(&mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, visit: Visit) {
        match self {
            Self::VigilanteGun(ability) => ability.on_visit_wardblocked(game, midnight_variables, actor_ref, visit),
        }
    }
    
    pub fn on_any_death(&mut self, game: &mut Game, actor_ref: PlayerReference, dead_player_ref: PlayerReference) {
        match self {
            Self::VigilanteGun(ability) => ability.on_any_death(game, actor_ref, dead_player_ref),
        }
    }
    
    pub fn on_grave_added(&mut self, game: &mut Game, actor_ref: PlayerReference, grave: GraveReference) {
        match self {
            Self::VigilanteGun(ability) => ability.on_grave_added(game, actor_ref, grave),
        }
    }
    
    pub fn on_game_start(&mut self, game: &mut Game, actor_ref: PlayerReference) {
        match self {
            Self::VigilanteGun(ability) => ability.on_game_start(game, actor_ref),
        }
    }
    
    pub fn on_game_ending(&mut self, game: &mut Game, actor_ref: PlayerReference) {
        match self {
            Self::VigilanteGun(ability) => ability.on_game_ending(game, actor_ref),
        }
    }
    
    pub fn removed_on_death(&self) -> bool {
        match self {
            Self::VigilanteGun(ability) => ability.removed_on_death(),
        }
    }
    
    pub fn is_active(&self) -> bool {
        match self {
            Self::VigilanteGun(ability) => ability.is_active(),
        }
    }
}

/// Storage for player abilities
pub type PlayerAbilities = HashMap<AbilityID, AbilityStateEnum>;
