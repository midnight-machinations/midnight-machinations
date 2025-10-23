use crate::{game::{abilities_component::{ability::Ability, ability_trait::{AbilityIDAndAbility, AbilityTrait, AbilityTraitOld}}, prelude::*}, impl_ability_events};

#[derive(Clone, Debug)]
pub struct RoleAbility(pub RoleState);
impl AbilityTraitOld for RoleAbility {
    fn on_ability_creation(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_ability_creation::OnAbilityCreation, fold: &mut crate::game::event::on_ability_creation::OnAbilityCreationFold, priority: crate::game::event::on_ability_creation::OnAbilityCreationPriority) {
        self.0.clone().on_ability_creation(game, id.get_role_actor_expect(), event, fold, priority)
    }
    fn on_ability_deletion(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_ability_deletion::OnAbilityDeletion, fold: &mut (), priority: crate::game::event::on_ability_deletion::OnAbilityDeletionPriority) {
        self.0.clone().on_ability_deletion(game, id.get_role_actor_expect(), event, fold, priority)
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
impl_ability_events!(AbilityIDAndAbility<RoleAbility>, BeforePhaseEnd, OnAddInsider, OnRemoveInsider);
impl EventListener<OnMidnight> for AbilityIDAndAbility<RoleAbility> {
    fn on_event(&self, game: &mut Game, _data: &OnMidnight, fold: &mut <OnMidnight as crate::game::event::EventData>::FoldValue, priority: <OnMidnight as crate::game::event::EventData>::Priority) {
        if priority == OnMidnightPriority::InitializeNight { 
            Visits::add_visits(
                fold,
                self.ability().0.clone().convert_selection_to_visits(game, self.id().get_role_actor_expect())
            );
        }
        
        self.ability().0.clone().on_midnight(game, self.id(), self.id().get_role_actor_expect(), fold, priority)
    }
}
impl EventListener<OnWhisper> for AbilityIDAndAbility<RoleAbility> {
    fn on_event(&self, game: &mut Game, data: &OnWhisper, fold: &mut <OnWhisper as crate::game::event::EventData>::FoldValue, priority: <OnWhisper as crate::game::event::EventData>::Priority) {
        self.ability().0.clone().on_whisper(game, self.id().get_role_actor_expect(), data, fold, priority);
    }
}
impl EventListener<OnGraveAdded> for AbilityIDAndAbility<RoleAbility> {
    fn on_event(&self, game: &mut Game, data: &OnGraveAdded, _fold: &mut <OnGraveAdded as EventData>::FoldValue, _priority: <OnGraveAdded as EventData>::Priority) {
        self.ability().0.clone().on_grave_added(game, self.id().get_role_actor_expect(), data.grave)
    }
}
impl EventListener<OnValidatedControllerInputReceived> for AbilityIDAndAbility<RoleAbility> {
    fn on_event(&self, game: &mut Game, data: &OnValidatedControllerInputReceived, _fold: &mut <OnValidatedControllerInputReceived as EventData>::FoldValue, _priority: <OnValidatedControllerInputReceived as EventData>::Priority) {
        self.ability().0.clone().on_validated_ability_input_received(game, self.id().get_role_actor_expect(), data.actor_ref, data.input.clone())
    }
}
impl EventListener<OnAnyDeath> for AbilityIDAndAbility<RoleAbility> {
    fn on_event(&self, game: &mut Game, data: &OnAnyDeath, _fold: &mut <OnAnyDeath as EventData>::FoldValue, _priority: <OnAnyDeath as EventData>::Priority) {
        self.ability().0.clone().on_any_death(game, self.id().get_role_actor_expect(), data.dead_player);
    }
}
impl EventListener<OnControllerSelectionChanged> for AbilityIDAndAbility<RoleAbility> {
    fn on_event(&self, game: &mut Game, data: &OnControllerSelectionChanged, _fold: &mut <OnControllerSelectionChanged as EventData>::FoldValue, _priority: <OnControllerSelectionChanged as EventData>::Priority) {
        self.ability().0.clone().on_controller_selection_changed(game, self.id().get_role_actor_expect(), data.id.clone())
    }
}
impl EventListener<OnPhaseStart> for AbilityIDAndAbility<RoleAbility> {
    fn on_event(&self, game: &mut Game, data: &OnPhaseStart, _fold: &mut <OnPhaseStart as EventData>::FoldValue, _priority: <OnPhaseStart as EventData>::Priority) {
        self.ability().0.clone().on_phase_start(game, self.id().get_role_actor_expect(), data.phase.phase())
    }
}
impl EventListener<OnConcealRole> for AbilityIDAndAbility<RoleAbility> {
    fn on_event(&self, game: &mut Game, data: &OnConcealRole, _fold: &mut <OnConcealRole as EventData>::FoldValue, _priority: <OnConcealRole as EventData>::Priority) {
        let &OnConcealRole{player: event_player, concealed_player} = data;
        self.ability().0.clone().on_conceal_role(game, self.id().get_role_actor_expect(), event_player, concealed_player)
    }
}
impl EventListener<OnRoleSwitch> for AbilityIDAndAbility<RoleAbility> {
    fn on_event(&self, game: &mut Game, data: &OnRoleSwitch, fold: &mut <OnRoleSwitch as EventData>::FoldValue, priority: <OnRoleSwitch as EventData>::Priority) {
        self.ability().0.clone().on_role_switch(game, self.id().get_role_actor_expect(), data, fold, priority)
    }
}


impl From<RoleAbility> for Ability where RoleAbility: AbilityTraitOld {
    fn from(role_struct: RoleAbility) -> Self {
        Ability::Role(role_struct)
    }
}