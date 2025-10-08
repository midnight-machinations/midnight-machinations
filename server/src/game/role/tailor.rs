use serde::Serialize;
use crate::game::components::night_visits::Visits;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::attack_power::DefensePower;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;
use crate::game::role::GetClientAbilityState;
use crate::game::role_list::RoleSet;
use crate::game::visit::Visit;
use crate::game::Game;
use super::{
    ControllerID,
    ControllerParametersMap,
    Role, RoleStateTrait
};
use crate::game::abilities_component::ability_id::AbilityID;


#[derive(Clone, Debug, Default)]
pub struct Tailor{
    previous_target: Option<PlayerReference>
}
#[derive(Clone, Debug, Serialize)]
pub struct ClientRoleState;
impl GetClientAbilityState<ClientRoleState> for Tailor {
    fn get_client_ability_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState
    }
}


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Tailor {
    type ClientAbilityState = ClientRoleState;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Convert {return;}
        let Some(target) = Visits::default_target(game, midnight_variables, actor_ref) else {return};
        let Some(role) = ControllerID::role(actor_ref, Role::Tailor, 1).get_role_list_selection_first(game) else {return};
    
        if !RoleSet::TownCommon.get_roles().contains(&target.role(game)) {return}
        target.set_night_convert_role_to(midnight_variables, Some(role.new_state(game)));
        actor_ref.edit_role_ability_helper(game, Tailor{previous_target: Some(target)});
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, phase: crate::game::phase::PhaseType) {
        if matches!(phase, PhaseType::Obituary){
            if let Some(target) = self.previous_target {
                actor_ref.reveal_players_role(game, target);
            }
            actor_ref.edit_role_ability_helper(game, Tailor{previous_target: None});
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::combine([
            //player to convert
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Tailor, 0))
                .single_player_selection_typical(actor_ref, false, true)
                .night_typical(actor_ref)
                .build_map(),
            //role
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Tailor, 1))
                .single_role_selection_typical(game, |r|RoleSet::TownCommon.get_roles().contains(r))
                .night_typical(actor_ref)
                .build_map()
        ])
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        if ControllerID::role(actor_ref, Role::Tailor, 1).get_role_list_selection_first(game).is_none() {return Vec::new()}
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Tailor, 0),
            false,
        )
    }
}