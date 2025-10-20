use crate::game::{
        abilities_component::{
            ability::Ability,
            ability_id::AbilityID,
            ability_trait::AbilityTrait
        }, components::{night_visits::Visits, possession::Possession}, controllers::ControllerID, event::{on_conceal_role::OnConcealRole, on_midnight::{OnMidnightFold, OnMidnightPriority}, on_player_possessed::OnPlayerPossessed, on_player_roleblocked::OnPlayerRoleblocked, on_visit_wardblocked::OnVisitWardblocked}, player::PlayerReference, role::RoleState, visit::VisitTag, Game
    };

#[derive(Clone, Debug)]
pub struct RoleAbility(pub RoleState);
impl AbilityTrait for RoleAbility {
    fn on_midnight(&self, game: &mut Game, id: &AbilityID, _event: &crate::game::event::on_midnight::OnMidnight, midnight_variables: &mut crate::game::event::on_midnight::OnMidnightFold, priority: crate::game::event::on_midnight::OnMidnightPriority) {
        if priority == OnMidnightPriority::InitializeNight { 
            Visits::add_visits(
                midnight_variables,
                self.0.clone().convert_selection_to_visits(game, id.get_role_actor_expect())
            );
        }
        
        self.0.clone().on_midnight(game, id, id.get_role_actor_expect(), midnight_variables, priority)
    }
    fn on_whisper(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_whisper::OnWhisper, fold: &mut crate::game::event::on_whisper::WhisperFold, priority: crate::game::event::on_whisper::WhisperPriority) {
        self.0.clone().on_whisper(game, id.get_role_actor_expect(), event, fold, priority);
    }
    fn on_grave_added(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_grave_added::OnGraveAdded, __fold: &mut (), __priority: ()) {
        self.0.clone().on_grave_added(game, id.get_role_actor_expect(), event.grave);
    }
    fn on_validated_ability_input_received(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_validated_ability_input_received::OnValidatedControllerInputReceived, _fold: &mut (), _priority: ()) {
        self.0.clone().on_validated_ability_input_received(game, id.get_role_actor_expect(), event.actor_ref, event.input.clone())
    }
    fn on_controller_selection_changed(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_controller_selection_changed::OnControllerSelectionChanged, _fold: &mut (), __priority: ()) {
        self.0.clone().on_controller_selection_changed(game, id.get_role_actor_expect(), event.id.clone())
    }
    fn on_phase_start(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_phase_start::OnPhaseStart, __fold: &mut (), __priority: ()) {
        self.0.clone().on_phase_start(game, id.get_role_actor_expect(), event.phase.phase())
    }
    fn on_conceal_role(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_conceal_role::OnConcealRole, __fold: &mut (), __priority: ()) {
        let &OnConcealRole{player: event_player, concealed_player} = event;
        self.0.clone().on_conceal_role(game, id.get_role_actor_expect(), event_player, concealed_player)
    }
    fn on_any_death(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_any_death::OnAnyDeath, _fold: &mut (), _priority: ()) {
        self.0.clone().on_any_death(game, id.get_role_actor_expect(), event.dead_player);
    }
    fn on_ability_creation(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_ability_creation::OnAbilityCreation, fold: &mut crate::game::event::on_ability_creation::OnAbilityCreationFold, priority: crate::game::event::on_ability_creation::OnAbilityCreationPriority) {
        self.0.clone().on_ability_creation(game, id.get_role_actor_expect(), event, fold, priority)
    }
    fn on_ability_deletion(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_ability_deletion::OnAbilityDeletion, fold: &mut (), priority: crate::game::event::on_ability_deletion::OnAbilityDeletionPriority) {
        self.0.clone().on_ability_deletion(game, id.get_role_actor_expect(), event, fold, priority)
    }
    fn on_role_switch(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_role_switch::OnRoleSwitch, fold: &mut (), priority: ()) {
        self.0.clone().on_role_switch(game, id.get_role_actor_expect(), event, fold, priority)
    }


    fn on_player_possessed(&self, game: &mut Game, id: &AbilityID, event: &OnPlayerPossessed, fold: &mut OnMidnightFold, priority: ()){
        if id.get_role_actor_expect() == event.possessed {
            for id in game.controllers.all_controller_ids() {
                if let ControllerID::Role { role, .. } = id && role == self.0.role() {
                    if Possession::possession_immune(&id) { continue; }
                    Possession::possess_controller(game, id.clone(), event.possessed, event.possessed_into)
                }
            }
            
            Visits::retain(fold, |v|
                if let VisitTag::Role { role, .. } = v.tag {role != self.0.role()} else { true }
            );
            Visits::add_visits(
                fold,
                self.0.clone().convert_selection_to_visits(game, id.get_role_actor_expect())
            );
        }

        self.0.clone().on_player_possessed(game, id, event, fold, priority);
    }


    fn on_player_roleblocked(&self, game: &mut Game, id: &AbilityID, event: &OnPlayerRoleblocked, fold: &mut OnMidnightFold, _priority: ()) {
        self.0.clone().on_player_roleblocked(game, fold, id.get_role_actor_expect(), event.player, event.invisible)
    }
    fn on_visit_wardblocked(&self, game: &mut Game, id: &AbilityID, event: &OnVisitWardblocked, fold: &mut OnMidnightFold, _priority: ()) {
        self.0.clone().on_visit_wardblocked(game, fold, id.get_role_actor_expect(), event.visit)
    }

    fn controller_parameters_map(&self, game: &Game, id: &AbilityID)  -> crate::game::controllers::ControllerParametersMap {
        self.0.clone().controller_parameters_map(game, id.get_role_actor_expect())
    }
}
impl AbilityID {
    fn get_role_actor_expect(&self)->PlayerReference {
        self.get_player_from_role_id().expect("RoleAbility event should only be called with a AbilityID::Role")
    }
    pub fn set_role_ability(&self, game: &mut Game, new: Option<impl Into<RoleState>>){
        self.set_ability(game, new.map(|o| RoleAbility(o.into())));
    }
    pub fn new_role_ability(&self, game: &mut Game, new: impl Into<RoleState>){
        self.new_ability(game, RoleAbility(new.into()));
    }
    pub fn edit_role_ability(&self, game: &mut Game, new: impl Into<RoleState>){
        self.edit_ability(game, RoleAbility(new.into()));
    }
}

impl From<RoleAbility> for Ability where RoleAbility: AbilityTrait {
    fn from(role_struct: RoleAbility) -> Self {
        Ability::Role(role_struct)
    }
}