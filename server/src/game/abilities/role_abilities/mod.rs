use crate::{
    game::{
        abilities_component::{
            ability::Ability,
            ability_id::AbilityID,
            ability_trait::AbilityTrait
        }, components::role::RoleComponent, event::on_conceal_role::OnConcealRole, player::PlayerReference, role::{Role, RoleState}, Game
    },
    packet::ToClientPacket
};

#[derive(Clone)]
pub struct RoleAbility(pub PlayerReference, pub RoleState);
impl AbilityTrait for RoleAbility {
    fn on_midnight(&self, game: &mut Game, id: &AbilityID, __event: &crate::game::event::on_midnight::OnMidnight, midnight_variables: &mut crate::game::event::on_midnight::MidnightVariables, priority: crate::game::event::on_midnight::OnMidnightPriority) {
        self.1.clone().on_midnight(game, midnight_variables, id.get_player_from_role_id_expect(), priority)
    }
    fn on_whisper(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_whisper::OnWhisper, fold: &mut crate::game::event::on_whisper::WhisperFold, priority: crate::game::event::on_whisper::WhisperPriority) {
        self.1.clone().on_whisper(game, id.get_player_from_role_id_expect(), event, fold, priority);
    }
    fn on_grave_added(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_grave_added::OnGraveAdded, __fold: &mut (), __priority: ()) {
        self.1.clone().on_grave_added(game, id.get_player_from_role_id_expect(), event.grave);
    }
    fn on_validated_ability_input_received(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_validated_ability_input_received::OnValidatedControllerInputReceived, _fold: &mut (), _priority: ()) {
        self.1.clone().on_validated_ability_input_received(game, id.get_player_from_role_id_expect(), event.actor_ref, event.input.clone())
    }
    fn on_controller_selection_changed(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_controller_selection_changed::OnControllerSelectionChanged, _fold: &mut (), __priority: ()) {
        self.1.clone().on_controller_selection_changed(game, id.get_player_from_role_id_expect(), event.id.clone())
    }
    fn on_phase_start(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_phase_start::OnPhaseStart, __fold: &mut (), __priority: ()) {
        self.1.clone().on_phase_start(game, id.get_player_from_role_id_expect(), event.phase.phase())
    }
    fn on_conceal_role(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_conceal_role::OnConcealRole, __fold: &mut (), __priority: ()) {
        let &OnConcealRole{player: event_player, concealed_player} = event;
        self.1.clone().on_conceal_role(game, id.get_player_from_role_id_expect(), event_player, concealed_player)
    }
    fn on_any_death(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_any_death::OnAnyDeath, _fold: &mut (), _priority: ()) {
        self.1.clone().on_any_death(game, id.get_player_from_role_id_expect(), event.dead_player);
    }
    fn on_ability_creation(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_ability_creation::OnAbilityCreation, fold: &mut crate::game::event::on_ability_creation::OnAbilityCreationFold, priority: crate::game::event::on_ability_creation::OnAbilityCreationPriority) {
        self.1.clone().on_ability_creation(game, id.get_player_from_role_id_expect(), event, fold, priority)
    }
    fn on_ability_deletion(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_ability_deletion::OnAbilityDeletion, fold: &mut (), priority: crate::game::event::on_ability_deletion::OnAbilityDeletionPriority) {
        self.1.clone().on_ability_deletion(game, id.get_player_from_role_id_expect(), event, fold, priority)
    }
    fn on_role_switch(&self, game: &mut Game, id: &AbilityID, event: &crate::game::event::on_role_switch::OnRoleSwitch, fold: &mut (), priority: ()) {
        self.1.clone().on_role_switch(game, id.get_player_from_role_id_expect(), event, fold, priority)
    }
    fn controller_parameters_map(&self, game: &Game, id: &AbilityID)  -> crate::game::controllers::ControllerParametersMap {
        self.1.clone().controller_parameters_map(game, id.get_player_from_role_id_expect())
    }
}
impl AbilityID {
    fn get_player_from_role_id_expect(&self)->PlayerReference {
        self.get_player_from_role_id().expect("RoleAbility event should be called with a AbilityID::Role")
    }
}

impl From<RoleAbility> for Ability where RoleAbility: AbilityTrait {
    fn from(role_struct: RoleAbility) -> Self {
        Ability::RoleAbility(role_struct)
    }
}
impl Role{
    pub fn should_inform_player_of_assignment(&self)->bool{
        !matches!(self, Role::Pawn|Role::Drunk)
    }
}

impl PlayerReference{
    pub fn role_state<'a>(&self, game: &'a Game) -> &'a RoleState {
        let Ability::RoleAbility(RoleAbility(_, role_state)) = AbilityID::Role { role: self.role(game), player: *self }
            .get_ability(game)
            .expect("every player must have a role ability")
            else { unreachable!() };
        
        role_state
    }
    pub fn set_role_state(&self, game: &mut Game, new_role_data: impl Into<RoleState>) {
        let new_role_data = new_role_data.into();
        let new_role = new_role_data.role();

        if self.role(game) != new_role {
            AbilityID::Role { role: self.role(game), player: *self }
                .delete_ability(game);
        }

        self.set_role_state_without_deleting_previous(game, new_role_data);
    }
    pub fn set_role_state_without_deleting_previous(&self, game: &mut Game, new_role_data: impl Into<RoleState>){
        let new_role_data = new_role_data.into();
        let new_role = new_role_data.role();
        RoleComponent::set_role(*self, game, new_role);

        AbilityID::Role { role: new_role, player: *self }
            .set_ability(game, Some(Ability::RoleAbility(RoleAbility(*self, new_role_data.clone()))));

        if self.role(game).should_inform_player_of_assignment() {
            self.send_packet(game, ToClientPacket::YourRoleState {
                role_state: new_role_data.get_client_ability_state(game, *self)
            });
        }
    }
}