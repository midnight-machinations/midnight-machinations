use crate::game::{
    abilities_component::ability_id::AbilityID, controllers::ControllerParametersMap,
    event::{
        before_phase_end::BeforePhaseEnd, on_ability_creation::{OnAbilityCreation, OnAbilityCreationFold, OnAbilityCreationPriority},
        on_ability_deletion::{OnAbilityDeletion, OnAbilityDeletionPriority}, on_add_insider::OnAddInsider, on_any_death::OnAnyDeath,
        on_conceal_role::OnConcealRole, on_controller_selection_changed::OnControllerSelectionChanged, on_grave_added::OnGraveAdded,
        on_midnight::{OnMidnight, OnMidnightFold, OnMidnightPriority}, on_phase_start::OnPhaseStart, on_player_possessed::OnPlayerPossessed,
        on_player_roleblocked::OnPlayerRoleblocked, on_remove_insider::OnRemoveInsider, on_role_switch::OnRoleSwitch,
        on_validated_ability_input_received::OnValidatedControllerInputReceived, on_visit_wardblocked::OnVisitWardblocked,
        on_whisper::{OnWhisper, WhisperFold, WhisperPriority}
    }, Game
};

pub trait AbilityTrait {
    fn on_midnight(&self, _game: &mut Game, _id: &AbilityID, __event: &OnMidnight, _midnight_variables: &mut OnMidnightFold, _priority: OnMidnightPriority) {}
    fn on_whisper(&self, _game: &mut Game, _id: &AbilityID, _event: &OnWhisper, _fold: &mut WhisperFold, _priority: WhisperPriority) {}
    fn on_grave_added(&self, _game: &mut Game, _id: &AbilityID, _event: &OnGraveAdded, __fold: &mut (), __priority: ()) {}
    fn on_validated_ability_input_received(&self, _game: &mut Game, _id: &AbilityID, _event: &OnValidatedControllerInputReceived, _fold: &mut (), _priority: ()) {}
    fn on_controller_selection_changed(&self, _game: &mut Game, _id: &AbilityID, _event: &OnControllerSelectionChanged, __fold: &mut (), __priority: ()) {}
    fn on_phase_start(&self, _game: &mut Game, _id: &AbilityID, _event: &OnPhaseStart, __fold: &mut (), __priority: ()) {}
    fn before_phase_end(&self, _game: &mut Game, _id: &AbilityID, _event: &BeforePhaseEnd, _fold: &mut (), _priority: ()) {}
    fn on_conceal_role(&self, _game: &mut Game, _id: &AbilityID, _event: &OnConcealRole, __fold: &mut (), __priority: ()) {}
    fn on_add_insider(&self, _game: &mut Game, _id: &AbilityID, _event: &OnAddInsider, _fold: &mut (), _priority: ()) {}
    fn on_remove_insider(&self, _game: &mut Game, _id: &AbilityID, _event: &OnRemoveInsider, _fold: &mut (), _priority: ()) {}
    fn on_any_death(&self, _game: &mut Game, _id: &AbilityID, _event: &OnAnyDeath, _fold: &mut (), _priority: ()) {}
    fn on_ability_creation(&self, _game: &mut Game, _id: &AbilityID, _event: &OnAbilityCreation, _fold: &mut OnAbilityCreationFold, _priority: OnAbilityCreationPriority) {}
    fn on_ability_deletion(&self, _game: &mut Game, _id: &AbilityID, _event: &OnAbilityDeletion, _fold: &mut (), _priority: OnAbilityDeletionPriority) {}
    fn on_role_switch(&self, _game: &mut Game, _id: &AbilityID, _event: &OnRoleSwitch, _fold: &mut (), _priority: ()) {}
    fn on_player_possessed(&self, _game: &mut Game, _id: &AbilityID, _event: &OnPlayerPossessed, _fold: &mut OnMidnightFold, _priority: ()) {}
    fn on_player_roleblocked(&self, _game: &mut Game, _id: &AbilityID, _event: &OnPlayerRoleblocked, _fold: &mut OnMidnightFold, _priority: ()) {}
    fn on_visit_wardblocked(&self, _game: &mut Game, _id: &AbilityID, _event: &OnVisitWardblocked, _fold: &mut OnMidnightFold, _priority: ()) {}

    fn controller_parameters_map(&self, _game: &Game, _id: &AbilityID)  -> ControllerParametersMap {ControllerParametersMap::default()}
}